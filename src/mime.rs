use gmime_sys;
use gobject_2_0_sys;
use libc::{c_char, c_void};
use magic::{Cookie, CookieFlags};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::ptr;
use std::slice;

extern "C" {
    fn new_mime_message(_msg: *mut gmime_sys::GMimeObject) -> *mut c_char;
    fn delete_mime_message(_msg: *mut c_char);
    fn mime_message_walk(
        _msg: *mut c_char,
        _cb: extern "C" fn(
            _data: *const c_char,
            _len: u32,
            _is_text: bool,
            _filename: *const c_char,
            _content: *mut gmime_sys::GMimeContentType,
            _user: *mut c_char,
        ),
        _user: *mut c_char,
    );
}

pub struct MimeParser {
    stream: *mut gmime_sys::GMimeStream,
    root_msg: *mut c_char,
    file_data_callback: Option<Box<FileDataCallback>>,
}

type FileDataCallback = Fn(&[u8], bool, String, String);

impl MimeParser {
    pub fn init() {
        unsafe { gmime_sys::g_mime_init() }
    }

    pub fn shutdown() {
        unsafe { gmime_sys::g_mime_shutdown() }
    }

    pub fn new(stream: *mut gmime_sys::GMimeStream) -> MimeParser {
        unsafe {
            gobject_2_0_sys::g_object_ref(stream as *mut c_void);
        }
        MimeParser {
            stream,
            root_msg: ptr::null_mut(),
            file_data_callback: None,
        }
    }

    pub fn parse(&mut self, callback: Box<FileDataCallback>) -> Result<(), ()> {
        self.file_data_callback = Some(callback);
        unsafe {
            let parser = gmime_sys::g_mime_parser_new_with_stream(self.stream);
            gmime_sys::g_mime_parser_set_format(parser, gmime_sys::GMIME_FORMAT_MESSAGE);

            let msg = gmime_sys::g_mime_parser_construct_message(
                parser,
                ptr::null_mut() as *mut gmime_sys::GMimeParserOptions,
            );
            if msg != ptr::null_mut() && (*msg).mime_part != ptr::null_mut() {
                self.root_msg = new_mime_message((*msg).mime_part);

                let this = self as *mut MimeParser as *mut c_char;

                mime_message_walk(self.root_msg, MimeParser::on_file_data, this);
            }

            if msg != ptr::null_mut() {
                gobject_2_0_sys::g_object_unref(msg as *mut c_void);
            }

            gobject_2_0_sys::g_object_unref(parser as *mut c_void);
        }

        Ok(())
    }

    fn on_file_data_callback(
        &self,
        data: &[u8],
        is_test: bool,
        filename: String,
        mime_type: String,
    ) {
        match self.file_data_callback.as_ref() {
            Some(cb) => {
                cb(data, is_test, filename, mime_type);
            }
            None => {}
        };
    }

    extern "C" fn on_file_data(
        data: *const c_char,
        len: u32,
        is_text: bool,
        filename: *const c_char,
        content: *mut gmime_sys::GMimeContentType,
        user: *mut c_char,
    ) {
        unsafe {
            let name;
            if filename != ptr::null_mut() {
                let filename = CStr::from_ptr(filename);
                name = filename.to_string_lossy().into_owned();
            } else {
                name = String::new();
            }

            let mime_type;
            if content != ptr::null_mut() {
                let raw = gmime_sys::g_mime_content_type_get_mime_type(content);
                mime_type = CString::from_raw(raw as *mut c_char)
                    .to_string_lossy()
                    .into_owned();
            } else {
                mime_type = String::new();
            }

            let this = &*(user as *mut MimeParser);
            let date = slice::from_raw_parts(data as *const u8, len as usize);
            this.on_file_data_callback(date, is_text, name, mime_type);
        }
    }
}

impl Drop for MimeParser {
    fn drop(&mut self) {
        unsafe {
            if self.root_msg != ptr::null_mut() {
                delete_mime_message(self.root_msg);
            }
            gobject_2_0_sys::g_object_unref(self.stream as *mut c_void);
        }
    }
}

thread_local! {
    pub static mime_cookie : Cookie = new_magic_cookie(magic::flags::MIME_TYPE);
    pub static compress_cookie : Cookie = new_magic_cookie(magic::flags::COMPRESS);
}

fn new_magic_cookie(flag: CookieFlags) -> Cookie {
    let cookie = Cookie::open(flag).unwrap();
    let databases: Vec<&str> = Vec::new();
    cookie.load(&databases).unwrap();
    return cookie;
}

fn magic_mime_buffer(data: &[u8]) -> Result<String, magic::MagicError> {
    return mime_cookie.with(|cookie| cookie.buffer(data));
}

fn magic_compress_buffer(data: &[u8]) -> Result<String, magic::MagicError> {
    return compress_cookie.with(|cookie| cookie.buffer(data));
}

pub fn magic_buffer(data: &[u8]) -> Option<&'static str> {
    let mime_type = magic_mime_buffer(data);
    if let Err(err) = mime_type {
        debug!("magic_mime_buffer error {}", err.desc);
        return None;
    }

    let mime_type = mime_type.unwrap();
    if mime_type != "application/x-dosexec" {
        let result = find_magic_type(&mime_type);
        if result.is_some() {
            return result;
        }
        return None;

    }

    let compress = magic_compress_buffer(data);
    if let Err(err) = compress {
        debug!("magic_compress_buffer error {}", err.desc);
        return None;
    }

    let compress = compress.unwrap();
    if compress.find("DLL").is_some() {
        return Some("dll");
    } else if compress.find("native").is_some() {
        return Some("sys");
    } else {
        return Some("exe");
    }

    return None;
}

pub fn find_magic_type(content_mime: &str) -> Option<&'static str> {
    let result = MAGIC_TYPE.get(content_mime);
    match result {
        Some(type_str) => {
            return Some(type_str);
        }
        None => {
            return None;
        }
    }
}

fn new_magic_type_map() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("application/andrew-inset", "ez");
    m.insert("application/applixware", "aw");
    m.insert("application/atom+xml", "atom");
    m.insert("application/atomcat+xml", "atomcat");
    m.insert("application/atomsvc+xml", "atomsvc");
    m.insert("application/ccxml+xml", "ccxml");
    m.insert("application/cdmi-capability", "cdmia");
    m.insert("application/cdmi-container", "cdmic");
    m.insert("application/cdmi-domain", "cdmid");
    m.insert("application/cdmi-object", "cdmio");
    m.insert("application/cdmi-queue", "cdmiq");
    m.insert("application/cu-seeme", "cu");
    m.insert("application/davmount+xml", "davmount");
    m.insert("application/dssc+der", "dssc");
    m.insert("application/dssc+xml", "xdssc");
    m.insert("application/ecmascript", "ecma");
    m.insert("application/emma+xml", "emma");
    m.insert("application/epub+zip", "epub");
    m.insert("application/exi", "exi");
    m.insert("application/font-tdpfr", "pfr");
    m.insert("application/hyperstudio", "stk");
    m.insert("application/ipfix", "ipfix");
    m.insert("application/java-archive", "jar");
    m.insert("application/java-serialized-object", "ser");
    m.insert("application/java-vm", "class");
    m.insert("application/javascript", "js");
    m.insert("application/x-javascript", "js");
    m.insert("application/json", "json");
    m.insert("application/lost+xml", "lostxml");
    m.insert("application/mac-binhex40", "hqx");
    m.insert("application/mac-compactpro", "cpt");
    m.insert("application/mads+xml", "mads");
    m.insert("application/marc", "mrc");
    m.insert("application/marcxml+xml", "mrcx");
    m.insert("application/mathematica", "mb");
    m.insert("application/mathml+xml", "mathml");
    m.insert("application/mbox", "mbox");
    m.insert("application/mediaservercontrol+xml", "mscml");
    m.insert("application/metalink4+xml", "meta4");
    m.insert("application/mets+xml", "mets");
    m.insert("application/mods+xml", "mods");
    m.insert("application/mp21", "mp21");
    m.insert("application/mp4", "mp4s");
    m.insert("application/msword", "doc");
    m.insert("application/mxf", "mxf");
    m.insert("application/oda", "oda");
    m.insert("application/oebps-package+xml", "opf");
    m.insert("application/ogg", "ogx");
    m.insert("application/onenote", "onetoc");
    m.insert("application/patch-ops-error+xml", "xer");
    m.insert("application/pdf", "pdf");
    m.insert("application/pgp-encrypted", "pgp");
    m.insert("application/pgp-signature", "asc");
    m.insert("application/pics-rules", "prf");
    m.insert("application/pkcs10", "p10");
    m.insert("application/pkcs7-mime", "p7m");
    m.insert("application/pkcs7-signature", "p7s");
    m.insert("application/pkcs8", "p8");
    m.insert("application/pkix-attr-cert", "ac");
    m.insert("application/pkix-cert", "cer");
    m.insert("application/pkix-crl", "crl");
    m.insert("application/pkix-pkipath", "pkipath");
    m.insert("application/pkixcmp", "pki");
    m.insert("application/pls+xml", "pls");
    m.insert("application/postscript", "ps");
    m.insert("application/prs.cww", "cww");
    m.insert("application/pskc+xml", "pskcxml");
    m.insert("application/rdf+xml", "rdf");
    m.insert("application/reginfo+xml", "rif");
    m.insert("application/relax-ng-compact-syntax", "rnc");
    m.insert("application/resource-lists+xml", "rl");
    m.insert("application/resource-lists-diff+xml", "rld");
    m.insert("application/rls-services+xml", "rs");
    m.insert("application/rsd+xml", "rsd");
    m.insert("application/rss+xml", "rss");
    m.insert("application/rtf", "rtf");
    m.insert("application/sbml+xml", "sbml");
    m.insert("application/scvp-cv-request", "scq");
    m.insert("application/scvp-cv-response", "scs");
    m.insert("application/scvp-vp-request", "spq");
    m.insert("application/scvp-vp-response", "spp");
    m.insert("application/sdp", "sdp");
    m.insert("application/set-payment-initiation", "setpay");
    m.insert("application/set-registration-initiation", "setreg");
    m.insert("application/shf+xml", "shf");
    m.insert("application/smil+xml", "smil");
    m.insert("application/sparql-query", "rq");
    m.insert("application/sparql-results+xml", "srx");
    m.insert("application/srgs", "gram");
    m.insert("application/srgs+xml", "grxml");
    m.insert("application/sru+xml", "sru");
    m.insert("application/ssml+xml", "ssml");
    m.insert("application/tei+xml", "teicorpus");
    m.insert("application/thraud+xml", "tfi");
    m.insert("application/timestamped-data", "tsd");
    m.insert("application/vnd.3gpp.pic-bw-large", "plb");
    m.insert("application/vnd.3gpp.pic-bw-small", "psb");
    m.insert("application/vnd.3gpp.pic-bw-var", "pvb");
    m.insert("application/vnd.3gpp2.tcap", "tcap");
    m.insert("application/vnd.3m.post-it-notes", "pwn");
    m.insert("application/vnd.accpac.simply.aso", "aso");
    m.insert("application/vnd.accpac.simply.imp", "imp");
    m.insert("application/vnd.acucobol", "acu");
    m.insert("application/vnd.acucorp", "atc");
    m.insert(
        "application/vnd.adobe.air-application-installer-package+zip",
        "air",
    );
    m.insert("application/vnd.adobe.fxp", "fxp");
    m.insert("application/vnd.adobe.xdp+xml", "xdp");
    m.insert("application/vnd.adobe.xfdf", "xfdf");
    m.insert("application/vnd.ahead.space", "ahead");
    m.insert("application/vnd.airzip.filesecure.azf", "azf");
    m.insert("application/vnd.airzip.filesecure.azs", "azs");
    m.insert("application/vnd.amazon.ebook", "azw");
    m.insert("application/vnd.americandynamics.acc", "acc");
    m.insert("application/vnd.amiga.ami", "ami");
    m.insert("application/vnd.android.package-archive", "apk");
    m.insert(
        "application/vnd.anser-web-certificate-issue-initiation",
        "cii",
    );
    m.insert("application/vnd.anser-web-funds-transfer-initiation", "fti");
    m.insert("application/vnd.antix.game-component", "atx");
    m.insert("application/vnd.apple.installer+xml", "mpkg");
    m.insert("application/vnd.apple.mpegurl", "m3u8");
    m.insert("application/vnd.aristanetworks.swi", "swi");
    m.insert("application/vnd.audiograph", "aep");
    m.insert("application/vnd.blueice.multipass", "mpm");
    m.insert("application/vnd.bmi", "bmi");
    m.insert("application/vnd.businessobjects", "rep");
    m.insert("application/vnd.chemdraw+xml", "cdxml");
    m.insert("application/vnd.chipnuts.karaoke-mmd", "mmd");
    m.insert("application/vnd.cinderella", "cdy");
    m.insert("application/vnd.claymore", "cla");
    m.insert("application/vnd.cloanto.rp9", "rp9");
    m.insert("application/vnd.clonk.c4group", "c4g");
    m.insert("application/vnd.cluetrust.cartomobile-config", "c11amc");
    m.insert("application/vnd.cluetrust.cartomobile-config-pkg", "c11amz");
    m.insert("application/vnd.commonspace", "csp");
    m.insert("application/vnd.contact.cmsg", "cdbcmsg");
    m.insert("application/vnd.cosmocaller", "cmc");
    m.insert("application/vnd.crick.clicker", "clkx");
    m.insert("application/vnd.crick.clicker.keyboard", "clkk");
    m.insert("application/vnd.crick.clicker.palette", "clkp");
    m.insert("application/vnd.crick.clicker.template", "clkt");
    m.insert("application/vnd.crick.clicker.wordbank", "clkw");
    m.insert("application/vnd.criticaltools.wbs+xml", "wbs");
    m.insert("application/vnd.ctc-posml", "pml");
    m.insert("application/vnd.cups-ppd", "ppd");
    m.insert("application/vnd.curl.car", "car");
    m.insert("application/vnd.curl.pcurl", "pcurl");
    m.insert("application/vnd.data-vision.rdz", "rdz");
    m.insert("application/vnd.denovo.fcselayout-link", "fe_launch");
    m.insert("application/vnd.dna", "dna");
    m.insert("application/vnd.dolby.mlp", "mlp");
    m.insert("application/vnd.dpgraph", "dpg");
    m.insert("application/vnd.dreamfactory", "dfac");
    m.insert("application/vnd.dvb.ait", "ait");
    m.insert("application/vnd.dvb.service", "svc");
    m.insert("application/vnd.dynageo", "geo");
    m.insert("application/vnd.ecowin.chart", "mag");
    m.insert("application/vnd.enliven", "nml");
    m.insert("application/vnd.epson.esf", "esf");
    m.insert("application/vnd.epson.msf", "msf");
    m.insert("application/vnd.epson.quickanime", "qam");
    m.insert("application/vnd.epson.salt", "slt");
    m.insert("application/vnd.epson.ssf", "ssf");
    m.insert("application/vnd.eszigno3+xml", "es3");
    m.insert("application/vnd.ezpix-album", "ez2");
    m.insert("application/vnd.ezpix-package", "ez3");
    m.insert("application/vnd.fdf", "fdf");
    m.insert("application/vnd.fdsn.mseed", "mseed");
    m.insert("application/vnd.fdsn.seed", "seed");
    m.insert("application/vnd.flographit", "gph");
    m.insert("application/vnd.fluxtime.clip", "ftc");
    m.insert("application/vnd.framemaker", "fm");
    m.insert("application/vnd.frogans.fnc", "fnc");
    m.insert("application/vnd.frogans.ltf", "ltf");
    m.insert("application/vnd.fsc.weblaunch", "fsc");
    m.insert("application/vnd.fujitsu.oasys", "oas");
    m.insert("application/vnd.fujitsu.oasys2", "oa2");
    m.insert("application/vnd.fujitsu.oasys3", "oa3");
    m.insert("application/vnd.fujitsu.oasysgp", "fg5");
    m.insert("application/vnd.fujitsu.oasysprs", "bh2");
    m.insert("application/vnd.fujixerox.ddd", "ddd");
    m.insert("application/vnd.fujixerox.docuworks", "xdw");
    m.insert("application/vnd.fujixerox.docuworks.binder", "xbd");
    m.insert("application/vnd.fuzzysheet", "fzs");
    m.insert("application/vnd.genomatix.tuxedo", "txd");
    m.insert("application/vnd.geogebra.file", "ggb");
    m.insert("application/vnd.geogebra.tool", "ggt");
    m.insert("application/vnd.geometry-explorer", "gex");
    m.insert("application/vnd.geonext", "gxt");
    m.insert("application/vnd.geoplan", "g2w");
    m.insert("application/vnd.geospace", "g3w");
    m.insert("application/vnd.gmx", "gmx");
    m.insert("application/vnd.google-earth.kml+xml", "kml");
    m.insert("application/vnd.google-earth.kmz", "kmz");
    m.insert("application/vnd.grafeq", "gqf");
    m.insert("application/vnd.groove-account", "gac");
    m.insert("application/vnd.groove-help", "ghf");
    m.insert("application/vnd.groove-identity-message", "gim");
    m.insert("application/vnd.groove-injector", "grv");
    m.insert("application/vnd.groove-tool-message", "gtm");
    m.insert("application/vnd.groove-tool-template", "tpl");
    m.insert("application/vnd.groove-vcard", "vcg");
    m.insert("application/vnd.hal+xml", "hal");
    m.insert("application/vnd.handheld-entertainment+xml", "zmm");
    m.insert("application/vnd.hbci", "hbci");
    m.insert("application/vnd.hhe.lesson-player", "les");
    m.insert("application/vnd.hp-hpgl", "hpgl");
    m.insert("application/vnd.hp-hpid", "hpid");
    m.insert("application/vnd.hp-hps", "hps");
    m.insert("application/vnd.hp-jlyt", "jlt");
    m.insert("application/vnd.hp-pcl", "pcl");
    m.insert("application/vnd.hp-pclxl", "pclxl");
    m.insert("application/vnd.hydrostatix.sof-data", "sfd-hdstx");
    m.insert("application/vnd.hzn-3d-crossword", "x3d");
    m.insert("application/vnd.ibm.minipay", "mpy");
    m.insert("application/vnd.ibm.modcap", "afp");
    m.insert("application/vnd.ibm.rights-management", "irm");
    m.insert("application/vnd.ibm.secure-container", "sc");
    m.insert("application/vnd.iccprofile", "icc");
    m.insert("application/vnd.igloader", "igl");
    m.insert("application/vnd.immervision-ivp", "ivp");
    m.insert("application/vnd.immervision-ivu", "ivu");
    m.insert("application/vnd.insors.igm", "igm");
    m.insert("application/vnd.intercon.formnet", "xpw");
    m.insert("application/vnd.intergeo", "i2g");
    m.insert("application/vnd.intu.qbo", "qbo");
    m.insert("application/vnd.intu.qfx", "qfx");
    m.insert("application/vnd.ipunplugged.rcprofile", "rcprofile");
    m.insert("application/vnd.irepository.package+xml", "irp");
    m.insert("application/vnd.is-xpr", "xpr");
    m.insert("application/vnd.isac.fcs", "fcs");
    m.insert("application/vnd.jam", "jam");
    m.insert("application/vnd.jcp.javame.midlet-rms", "rms");
    m.insert("application/vnd.jisp", "jisp");
    m.insert("application/vnd.joost.joda-archive", "joda");
    m.insert("application/vnd.kahootz", "ktz");
    m.insert("application/vnd.kde.karbon", "karbon");
    m.insert("application/vnd.kde.kchart", "chrt");
    m.insert("application/vnd.kde.kformula", "kfo");
    m.insert("application/vnd.kde.kivio", "flw");
    m.insert("application/vnd.kde.kontour", "kon");
    m.insert("application/vnd.kde.kpresenter", "kpr");
    m.insert("application/vnd.kde.kspread", "ksp");
    m.insert("application/vnd.kde.kword", "kwd");
    m.insert("application/vnd.kenameaapp", "htke");
    m.insert("application/vnd.kidspiration", "kia");
    m.insert("application/vnd.kinar", "knp");
    m.insert("application/vnd.koan", "skp");
    m.insert("application/vnd.kodak-descriptor", "sse");
    m.insert("application/vnd.las.las+xml", "lasxml");
    m.insert("application/vnd.llamagraphics.life-balance.desktop", "lbd");
    m.insert(
        "application/vnd.llamagraphics.life-balance.exchange+xml",
        "lbe",
    );
    m.insert("application/vnd.lotus-1-2-3", "123");
    m.insert("application/vnd.lotus-approach", "apr");
    m.insert("application/vnd.lotus-freelance", "pre");
    m.insert("application/vnd.lotus-notes", "nsf");
    m.insert("application/vnd.lotus-organizer", "org");
    m.insert("application/vnd.lotus-screencam", "scm");
    m.insert("application/vnd.lotus-wordpro", "lwp");
    m.insert("application/vnd.macports.portpkg", "portpkg");
    m.insert("application/vnd.mcd", "mcd");
    m.insert("application/vnd.medcalcdata", "mc1");
    m.insert("application/vnd.mediastation.cdkey", "cdkey");
    m.insert("application/vnd.mfer", "mwf");
    m.insert("application/vnd.mfmp", "mfm");
    m.insert("application/vnd.micrografx.flo", "flo");
    m.insert("application/vnd.micrografx.igx", "igx");
    m.insert("application/vnd.mif", "mif");
    m.insert("application/vnd.mobius.daf", "daf");
    m.insert("application/vnd.mobius.dis", "dis");
    m.insert("application/vnd.mobius.mbk", "mbk");
    m.insert("application/vnd.mobius.mqy", "mqy");
    m.insert("application/vnd.mobius.msl", "msl");
    m.insert("application/vnd.mobius.plc", "plc");
    m.insert("application/vnd.mobius.txf", "txf");
    m.insert("application/vnd.mophun.application", "mpn");
    m.insert("application/vnd.mophun.certificate", "mpc");
    m.insert("application/vnd.mozilla.xul+xml", "xul");
    m.insert("application/vnd.ms-artgalry", "cil");
    m.insert("application/vnd.ms-cab-compressed", "cab");
    m.insert("application/vnd.ms-excel", "xls");
    m.insert("application/vnd.ms-excel.addin.macroenabled.12", "xlam");
    m.insert(
        "application/vnd.ms-excel.sheet.binary.macroenabled.12",
        "xlsb",
    );
    m.insert("application/vnd.ms-excel.sheet.macroenabled.12", "xlsm");
    m.insert("application/vnd.ms-excel.template.macroenabled.12", "xltm");
    m.insert("application/vnd.ms-fontobject", "eot");
    m.insert("application/vnd.ms-htmlhelp", "chm");
    m.insert("application/vnd.ms-ims", "ims");
    m.insert("application/vnd.ms-lrm", "lrm");
    m.insert("application/vnd.ms-officetheme", "thmx");
    m.insert("application/vnd.ms-pki.seccat", "cat");
    m.insert("application/vnd.ms-pki.stl", "stl");
    m.insert("application/vnd.ms-powerpoint", "ppt");
    m.insert(
        "application/vnd.ms-powerpoint.addin.macroenabled.12",
        "ppam",
    );
    m.insert(
        "application/vnd.ms-powerpoint.presentation.macroenabled.12",
        "pptm",
    );
    m.insert(
        "application/vnd.ms-powerpoint.slide.macroenabled.12",
        "sldm",
    );
    m.insert(
        "application/vnd.ms-powerpoint.slideshow.macroenabled.12",
        "ppsm",
    );
    m.insert(
        "application/vnd.ms-powerpoint.template.macroenabled.12",
        "potm",
    );
    m.insert("application/vnd.ms-project", "mpp");
    m.insert("application/vnd.ms-word.document.macroenabled.12", "docm");
    m.insert("application/vnd.ms-word.template.macroenabled.12", "dotm");
    m.insert("application/vnd.ms-works", "wps");
    m.insert("application/vnd.ms-wpl", "wpl");
    m.insert("application/vnd.ms-xpsdocument", "xps");
    m.insert("application/vnd.mseq", "mseq");
    m.insert("application/vnd.musician", "mus");
    m.insert("application/vnd.muvee.style", "msty");
    m.insert("application/vnd.neurolanguage.nlu", "nlu");
    m.insert("application/vnd.noblenet-directory", "nnd");
    m.insert("application/vnd.noblenet-sealer", "nns");
    m.insert("application/vnd.noblenet-web", "nnw");
    m.insert("application/vnd.nokia.n-gage.data", "ngdat");
    m.insert("application/vnd.nokia.n-gage.symbian.install", "n-gage");
    m.insert("application/vnd.nokia.radio-preset", "rpst");
    m.insert("application/vnd.nokia.radio-presets", "rpss");
    m.insert("application/vnd.novadigm.edm", "edm");
    m.insert("application/vnd.novadigm.edx", "edx");
    m.insert("application/vnd.novadigm.ext", "ext");
    m.insert("application/vnd.oasis.opendocument.chart", "odc");
    m.insert("application/vnd.oasis.opendocument.chart-template", "otc");
    m.insert("application/vnd.oasis.opendocument.database", "odb");
    m.insert("application/vnd.oasis.opendocument.formula", "odf");
    m.insert(
        "application/vnd.oasis.opendocument.formula-template",
        "odft",
    );
    m.insert("application/vnd.oasis.opendocument.graphics", "odg");
    m.insert(
        "application/vnd.oasis.opendocument.graphics-template",
        "otg",
    );
    m.insert("application/vnd.oasis.opendocument.image", "odi");
    m.insert("application/vnd.oasis.opendocument.image-template", "oti");
    m.insert("application/vnd.oasis.opendocument.presentation", "odp");
    m.insert(
        "application/vnd.oasis.opendocument.presentation-template",
        "otp",
    );
    m.insert("application/vnd.oasis.opendocument.spreadsheet", "ods");
    m.insert(
        "application/vnd.oasis.opendocument.spreadsheet-template",
        "ots",
    );
    m.insert("application/vnd.oasis.opendocument.text", "odt");
    m.insert("application/vnd.oasis.opendocument.text-master", "odm");
    m.insert("application/vnd.oasis.opendocument.text-template", "ott");
    m.insert("application/vnd.oasis.opendocument.text-web", "oth");
    m.insert("application/vnd.olpc-sugar", "xo");
    m.insert("application/vnd.oma.dd2+xml", "dd2");
    m.insert("application/vnd.openofficeorg.extension", "oxt");
    m.insert(
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "pptx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.presentationml.slide",
        "sldx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.presentationml.slideshow",
        "ppsx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.presentationml.template",
        "potx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xlsx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.spreadsheetml.template",
        "xltx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "docx",
    );
    m.insert(
        "application/vnd.openxmlformats-officedocument.wordprocessingml.template",
        "dotx",
    );
    m.insert("application/vnd.osgeo.mapguide.package", "mgp");
    m.insert("application/vnd.osgi.dp", "dp");
    m.insert("application/vnd.palm", "pdb");
    m.insert("application/vnd.pawaafile", "paw");
    m.insert("application/vnd.pg.format", "str");
    m.insert("application/vnd.pg.osasli", "ei6");
    m.insert("application/vnd.picsel", "efif");
    m.insert("application/vnd.pmi.widget", "wg");
    m.insert("application/vnd.pocketlearn", "plf");
    m.insert("application/vnd.powerbuilder6", "pbd");
    m.insert("application/vnd.previewsystems.box", "box");
    m.insert("application/vnd.proteus.magazine", "mgz");
    m.insert("application/vnd.publishare-delta-tree", "qps");
    m.insert("application/vnd.pvi.ptid1", "ptid");
    m.insert("application/vnd.quark.quarkxpress", "qxd");
    m.insert("application/vnd.realvnc.bed", "bed");
    m.insert("application/vnd.recordare.musicxml", "mxl");
    m.insert("application/vnd.recordare.musicxml+xml", "musicxml");
    m.insert("application/vnd.rig.cryptonote", "cryptonote");
    m.insert("application/vnd.rim.cod", "cod");
    m.insert("application/vnd.rn-realmedia", "rm");
    m.insert("application/vnd.route66.link66+xml", "link66");
    m.insert("application/vnd.sailingtracker.track", "st");
    m.insert("application/vnd.seemail", "see");
    m.insert("application/vnd.sema", "sema");
    m.insert("application/vnd.semd", "semd");
    m.insert("application/vnd.semf", "semf");
    m.insert("application/vnd.shana.informed.formdata", "ifm");
    m.insert("application/vnd.shana.informed.formtemplate", "itp");
    m.insert("application/vnd.shana.informed.interchange", "iif");
    m.insert("application/vnd.shana.informed.package", "ipk");
    m.insert("application/vnd.simtech-mindmapper", "twd");
    m.insert("application/vnd.smaf", "mmf");
    m.insert("application/vnd.smart.teacher", "teacher");
    m.insert("application/vnd.solent.sdkm+xml", "sdkm");
    m.insert("application/vnd.spotfire.dxp", "dxp");
    m.insert("application/vnd.spotfire.sfs", "sfs");
    m.insert("application/vnd.stardivision.calc", "sdc");
    m.insert("application/vnd.stardivision.draw", "sda");
    m.insert("application/vnd.stardivision.impress", "sdd");
    m.insert("application/vnd.stardivision.math", "smf");
    m.insert("application/vnd.stardivision.writer", "sdw");
    m.insert("application/vnd.stardivision.writer-global", "sgl");
    m.insert("application/vnd.stepmania.stepchart", "sm");
    m.insert("application/vnd.sun.xml.calc", "sxc");
    m.insert("application/vnd.sun.xml.calc.template", "stc");
    m.insert("application/vnd.sun.xml.draw", "sxd");
    m.insert("application/vnd.sun.xml.draw.template", "std");
    m.insert("application/vnd.sun.xml.impress", "sxi");
    m.insert("application/vnd.sun.xml.impress.template", "sti");
    m.insert("application/vnd.sun.xml.math", "sxm");
    m.insert("application/vnd.sun.xml.writer", "sxw");
    m.insert("application/vnd.sun.xml.writer.global", "sxg");
    m.insert("application/vnd.sun.xml.writer.template", "stw");
    m.insert("application/vnd.sus-calendar", "sus");
    m.insert("application/vnd.svd", "svd");
    m.insert("application/vnd.symbian.install", "sis");
    m.insert("application/vnd.syncml+xml", "xsm");
    m.insert("application/vnd.syncml.dm+wbxml", "bdm");
    m.insert("application/vnd.syncml.dm+xml", "xdm");
    m.insert("application/vnd.tao.intent-module-archive", "tao");
    m.insert("application/vnd.tmobile-livetv", "tmo");
    m.insert("application/vnd.trid.tpt", "tpt");
    m.insert("application/vnd.triscape.mxs", "mxs");
    m.insert("application/vnd.trueapp", "tra");
    m.insert("application/vnd.ufdl", "ufdl");
    m.insert("application/vnd.uiq.theme", "utz");
    m.insert("application/vnd.umajin", "umj");
    m.insert("application/vnd.unity", "unityweb");
    m.insert("application/vnd.uoml+xml", "uoml");
    m.insert("application/vnd.vcx", "vcx");
    m.insert("application/vnd.visio", "vsd");
    m.insert("application/vnd.visionary", "vis");
    m.insert("application/vnd.vsf", "vsf");
    m.insert("application/vnd.wap.wbxml", "wbxml");
    m.insert("application/vnd.wap.wmlc", "wmlc");
    m.insert("application/vnd.wap.wmlscriptc", "wmlsc");
    m.insert("application/vnd.webturbo", "wtb");
    m.insert("application/vnd.wolfram.player", "nbp");
    m.insert("application/vnd.wordperfect", "wpd");
    m.insert("application/vnd.wqd", "wqd");
    m.insert("application/vnd.wt.stf", "stf");
    m.insert("application/vnd.xara", "xar");
    m.insert("application/vnd.xfdl", "xfdl");
    m.insert("application/vnd.yamaha.hv-dic", "hvd");
    m.insert("application/vnd.yamaha.hv-script", "hvs");
    m.insert("application/vnd.yamaha.hv-voice", "hvp");
    m.insert("application/vnd.yamaha.openscoreformat", "osf");
    m.insert(
        "application/vnd.yamaha.openscoreformat.osfpvg+xml",
        "osfpvg",
    );
    m.insert("application/vnd.yamaha.smaf-audio", "saf");
    m.insert("application/vnd.yamaha.smaf-phrase", "spf");
    m.insert("application/vnd.yellowriver-custom-menu", "cmp");
    m.insert("application/vnd.zul", "zir");
    m.insert("application/vnd.zzazz.deck+xml", "zaz");
    m.insert("application/voicexml+xml", "vxml");
    m.insert("application/widget", "wgt");
    m.insert("application/winhlp", "hlp");
    m.insert("application/wsdl+xml", "wsdl");
    m.insert("application/wspolicy+xml", "wspolicy");
    m.insert("application/x-7z-compressed", "7z");
    m.insert("application/x-abiword", "abw");
    m.insert("application/x-ace-compressed", "ace");
    m.insert("application/x-authorware-map", "aam");
    m.insert("application/x-authorware-seg", "aas");
    m.insert("application/x-bcpio", "bcpio");
    m.insert("application/x-bittorrent", "torrent");
    m.insert("application/x-bzip", "bz");
    m.insert("application/x-bzip2", "bz2");
    m.insert("application/x-cdlink", "vcd");
    m.insert("application/x-chat", "chat");
    m.insert("application/x-chess-pgn", "pgn");
    m.insert("application/x-cpio", "cpio");
    m.insert("application/x-csh", "csh");
    m.insert("application/x-debian-package", "deb");
    m.insert("application/x-director", "dir");
    m.insert("application/x-doom", "wad");
    m.insert("application/x-dtbncx+xml", "ncx");
    m.insert("application/x-dtbook+xml", "dtb");
    m.insert("application/x-dtbresource+xml", "res");
    m.insert("application/x-dvi", "dvi");
    m.insert("application/x-font-bdf", "bdf");
    m.insert("application/x-font-ghostscript", "gsf");
    m.insert("application/x-font-linux-psf", "psf");
    m.insert("application/x-font-otf", "otf");
    m.insert("application/x-font-pcf", "pcf");
    m.insert("application/x-font-snf", "snf");
    m.insert("application/x-font-ttf", "ttf");
    m.insert("application/x-font-type1", "afm");
    m.insert("application/x-font-woff", "woff");
    m.insert("application/x-futuresplash", "spl");
    m.insert("application/x-gnumeric", "gnumeric");
    m.insert("application/x-gtar", "gtar");
    m.insert("application/x-hdf", "hdf");
    m.insert("application/x-java-jnlp-file", "jnlp");
    m.insert("application/x-latex", "latex");
    m.insert("application/x-mobipocket-ebook", "mobi");
    m.insert("application/x-mpegurl", "m3u8");
    m.insert("application/x-ms-application", "application");
    m.insert("application/x-ms-wmd", "wmd");
    m.insert("application/x-ms-wmz", "wmz");
    m.insert("application/x-ms-xbap", "xbap");
    m.insert("application/x-msaccess", "mdb");
    m.insert("application/x-msbinder", "obd");
    m.insert("application/x-mscardfile", "crd");
    m.insert("application/x-msclip", "clp");
    m.insert("application/x-msmediaview", "mvb");
    m.insert("application/x-msmetafile", "wmf");
    m.insert("application/x-msmoney", "mny");
    m.insert("application/x-mspublisher", "pub");
    m.insert("application/x-msschedule", "scd");
    m.insert("application/x-msterminal", "trm");
    m.insert("application/x-mswrite", "wri");
    m.insert("application/x-netcdf", "nc");
    m.insert("application/x-pkcs12", "p12");
    m.insert("application/x-pkcs7-certificates", "p7b");
    m.insert("application/x-pkcs7-certreqresp", "p7r");
    m.insert("application/x-rar-compressed", "rar");
    m.insert("application/x-sh", "sh");
    m.insert("application/x-shar", "shar");
    m.insert("application/x-shockwave-flash", "swf");
    m.insert("application/x-silverlight-app", "xap");
    m.insert("application/x-stuffit", "sit");
    m.insert("application/x-stuffitx", "sitx");
    m.insert("application/x-sv4cpio", "sv4cpio");
    m.insert("application/x-sv4crc", "sv4crc");
    m.insert("application/x-tar", "tar");
    m.insert("application/x-tcl", "tcl");
    m.insert("application/x-tex", "tex");
    m.insert("application/x-tex-tfm", "tfm");
    m.insert("application/x-texinfo", "texi");
    m.insert("application/x-ustar", "ustar");
    m.insert("application/x-wais-source", "src");
    m.insert("application/x-x509-ca-cert", "crt");
    m.insert("application/x-xfig", "fig");
    m.insert("application/x-xpinstall", "xpi");
    m.insert("application/xcap-diff+xml", "xdf");
    m.insert("application/xenc+xml", "xenc");
    m.insert("application/xhtml+xml", "xhtml");
    m.insert("application/xml", "xml");
    m.insert("application/xml-dtd", "dtd");
    m.insert("application/xop+xml", "xop");
    m.insert("application/xslt+xml", "xslt");
    m.insert("application/xspf+xml", "xspf");
    m.insert("application/xv+xml", "xvml");
    m.insert("application/yang", "yang");
    m.insert("application/yin+xml", "yin");
    m.insert("application/zip", "zip");
    m.insert("audio/adpcm", "adp");
    m.insert("audio/basic", "au");
    m.insert("audio/midi", "mid");
    m.insert("audio/mp4", "mp4a");
    m.insert("audio/mp4a-latm", "m4a");
    m.insert("audio/mpeg", "mpga");
    m.insert("audio/ogg", "ogg");
    m.insert("audio/vnd.dece.audio", "uvva");
    m.insert("audio/vnd.digital-winds", "eol");
    m.insert("audio/vnd.dra", "dra");
    m.insert("audio/vnd.dts", "dts");
    m.insert("audio/vnd.dts.hd", "dtshd");
    m.insert("audio/vnd.lucent.voice", "lvp");
    m.insert("audio/vnd.ms-playready.media.pya", "pya");
    m.insert("audio/vnd.nuera.ecelp4800", "ecelp4800");
    m.insert("audio/vnd.nuera.ecelp7470", "ecelp7470");
    m.insert("audio/vnd.nuera.ecelp9600", "ecelp9600");
    m.insert("audio/vnd.rip", "rip");
    m.insert("audio/webm", "weba");
    m.insert("audio/x-aac", "aac");
    m.insert("audio/x-aiff", "aiff");
    m.insert("audio/x-mpegurl", "m3u");
    m.insert("audio/x-ms-wax", "wax");
    m.insert("audio/x-ms-wma", "wma");
    m.insert("audio/x-pn-realaudio", "ram");
    m.insert("audio/x-pn-realaudio-plugin", "rmp");
    m.insert("audio/x-wav", "wav");
    m.insert("chemical/x-cdx", "cdx");
    m.insert("chemical/x-cif", "cif");
    m.insert("chemical/x-cmdf", "cmdf");
    m.insert("chemical/x-cml", "cml");
    m.insert("chemical/x-csml", "csml");
    m.insert("chemical/x-xyz", "xyz");
    m.insert("image/bmp", "bmp");
    m.insert("image/cgm", "cgm");
    m.insert("image/g3fax", "g3");
    m.insert("image/gif", "gif");
    m.insert("image/ief", "ief");
    m.insert("image/jp2", "jp2");
    m.insert("image/jpeg", "jpg");
    m.insert("image/ktx", "ktx");
    m.insert("image/pict", "pict");
    m.insert("image/png", "png");
    m.insert("image/prs.btif", "btif");
    m.insert("image/svg+xml", "svg");
    m.insert("image/tiff", "tiff");
    m.insert("image/vnd.adobe.photoshop", "psd");
    m.insert("image/vnd.dece.graphic", "uvi");
    m.insert("image/vnd.djvu", "djvu");
    m.insert("image/vnd.dvb.subtitle", "sub");
    m.insert("image/vnd.dwg", "dwg");
    m.insert("image/vnd.dxf", "dxf");
    m.insert("image/vnd.fastbidsheet", "fbs");
    m.insert("image/vnd.fpx", "fpx");
    m.insert("image/vnd.fst", "fst");
    m.insert("image/vnd.fujixerox.edmics-mmr", "mmr");
    m.insert("image/vnd.fujixerox.edmics-rlc", "rlc");
    m.insert("image/vnd.ms-modi", "mdi");
    m.insert("image/vnd.net-fpx", "npx");
    m.insert("image/vnd.wap.wbmp", "wbmp");
    m.insert("image/vnd.xiff", "xif");
    m.insert("image/webp", "webp");
    m.insert("image/x-cmu-raster", "ras");
    m.insert("image/x-cmx", "cmx");
    m.insert("image/x-freehand", "fh");
    m.insert("image/x-icon", "ico");
    m.insert("image/x-macpaint", "pntg");
    m.insert("image/x-pcx", "pcx");
    m.insert("image/x-pict", "pict");
    m.insert("image/x-portable-anymap", "pnm");
    m.insert("image/x-portable-bitmap", "pbm");
    m.insert("image/x-portable-graymap", "pgm");
    m.insert("image/x-portable-pixmap", "ppm");
    m.insert("image/x-quicktime", "qtif");
    m.insert("image/x-rgb", "rgb");
    m.insert("image/x-xbitmap", "xbm");
    m.insert("image/x-xpixmap", "xpm");
    m.insert("image/x-xwindowdump", "xwd");
    m.insert("message/rfc822", "eml");
    m.insert("model/iges", "iges");
    m.insert("model/mesh", "mesh");
    m.insert("model/vnd.collada+xml", "dae");
    m.insert("model/vnd.dwf", "dwf");
    m.insert("model/vnd.gdl", "gdl");
    m.insert("model/vnd.gtw", "gtw");
    m.insert("model/vnd.mts", "mts");
    m.insert("model/vnd.vtu", "vtu");
    m.insert("model/vrml", "vrml");
    m.insert("text/cache-manifest", "manifest");
    m.insert("text/calendar", "ics");
    m.insert("text/css", "css");
    m.insert("text/csv", "csv");
    m.insert("text/html", "html");
    m.insert("text/n3", "n3");
    m.insert("text/plain", "txt");
    m.insert("text/prs.lines.tag", "dsc");
    m.insert("text/richtext", "rtx");
    m.insert("text/sgml", "sgml");
    m.insert("text/tab-separated-values", "tsv");
    m.insert("text/troff", "roff");
    m.insert("text/turtle", "ttl");
    m.insert("text/uri-list", "urls");
    m.insert("text/vnd.curl", "curl");
    m.insert("text/vnd.curl.dcurl", "dcurl");
    m.insert("text/vnd.curl.mcurl", "mcurl");
    m.insert("text/vnd.curl.scurl", "scurl");
    m.insert("text/vnd.fly", "fly");
    m.insert("text/vnd.fmi.flexstor", "flx");
    m.insert("text/vnd.graphviz", "gv");
    m.insert("text/vnd.in3d.3dml", "3dml");
    m.insert("text/vnd.in3d.spot", "spot");
    m.insert("text/vnd.sun.j2me.app-descriptor", "jad");
    m.insert("text/vnd.wap.wml", "wml");
    m.insert("text/vnd.wap.wmlscript", "wmls");
    m.insert("text/x-asm", "asm");
    m.insert("text/x-c", "c");
    m.insert("text/x-fortran", "f");
    m.insert("text/x-java-source", "java");
    m.insert("text/x-pascal", "pas");
    m.insert("text/x-setext", "etx");
    m.insert("text/x-uuencode", "uu");
    m.insert("text/x-vcalendar", "vcs");
    m.insert("text/x-vcard", "vcf");
    m.insert("video/3gpp", "3gp");
    m.insert("video/3gpp2", "3g2");
    m.insert("video/h261", "h261");
    m.insert("video/h263", "h263");
    m.insert("video/h264", "h264");
    m.insert("video/jpeg", "jpgv");
    m.insert("video/jpm", "jpm");
    m.insert("video/mj2", "mj2");
    m.insert("video/mp2t", "ts");
    m.insert("video/mp4", "m4v");
    m.insert("video/mpeg", "mpg");
    m.insert("video/ogg", "ogv");
    m.insert("video/quicktime", "mov");
    m.insert("video/vnd.dece.hd", "uvvh");
    m.insert("video/vnd.dece.mobile", "uvvm");
    m.insert("video/vnd.dece.pd", "uvvp");
    m.insert("video/vnd.dece.sd", "uvvs");
    m.insert("video/vnd.dece.video", "uvvv");
    m.insert("video/vnd.fvt", "fvt");
    m.insert("video/vnd.mpegurl", "m4u");
    m.insert("video/vnd.ms-playready.media.pyv", "pyv");
    m.insert("video/vnd.uvvu.mp4", "uvvu");
    m.insert("video/vnd.vivo", "viv");
    m.insert("video/webm", "webm");
    m.insert("video/x-dv", "dv");
    m.insert("video/x-f4v", "f4v");
    m.insert("video/x-fli", "fli");
    m.insert("video/x-flv", "flv");
    m.insert("video/x-m4v", "m4v");
    m.insert("video/x-ms-asf", "asf");
    m.insert("video/x-ms-wm", "wm");
    m.insert("video/x-ms-wmv", "wmv");
    m.insert("video/x-ms-wmx", "wmx");
    m.insert("video/x-ms-wvx", "wvx");
    m.insert("video/x-msvideo", "avi");
    m.insert("video/x-sgi-movie", "movie");
    m.insert("x-conference/x-cooltalk", "ice");
    m.insert("application/x-rpm", "rpm");
    return m;
}

lazy_static! {
    static ref MAGIC_TYPE: HashMap<&'static str, &'static str> = new_magic_type_map();
}
