use libc::c_char;
use std::ffi::CStr;
use std::sync::Arc;
use config::Configure;
use layer::tcp::dissector::{TCPDissectorAllocator, TCPDissector};
use std::rc::Rc;
use std::cell::RefCell;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Proto {
    pub master_id: u16,
    pub app_id: u16,
    pub category: u32,
}

impl Proto {
    pub const UNKNOWN: u16 = 0;
    pub const FTP_CONTROL: u16 = 1;
    pub const MAIL_POP: u16 = 2;
    pub const MAIL_SMTP: u16 = 3;
    pub const MAIL_IMAP: u16 = 4;
    pub const DNS: u16 = 5;
    pub const IPP: u16 = 6;
    pub const HTTP: u16 = 7;
    pub const MDNS: u16 = 8;
    pub const NTP: u16 = 9;
    pub const NETBIOS: u16 = 10;
    pub const NFS: u16 = 11;
    pub const SSDP: u16 = 12;
    pub const BGP: u16 = 13;
    pub const SNMP: u16 = 14;
    pub const XDMCP: u16 = 15;
    pub const SMB: u16 = 16;
    pub const SYSLOG: u16 = 17;
    pub const DHCP: u16 = 18;
    pub const POSTGRES: u16 = 19;
    pub const MYSQL: u16 = 20;
    pub const HOTMAIL: u16 = 21;
    pub const DIRECT_DOWNLOAD_LINK: u16 = 22;
    pub const MAIL_POPS: u16 = 23;
    pub const APPLEJUICE: u16 = 24;
    pub const DIRECTCONNECT: u16 = 25;
    pub const NTOP: u16 = 26;
    pub const COAP: u16 = 27;
    pub const VMWARE: u16 = 28;
    pub const MAIL_SMTPS: u16 = 29;
    pub const FBZERO: u16 = 30;
    pub const UBNTAC2: u16 = 31;
    pub const KONTIKI: u16 = 32;
    pub const OPENFT: u16 = 33;
    pub const FASTTRACK: u16 = 34;
    pub const GNUTELLA: u16 = 35;
    pub const EDONKEY: u16 = 36;
    pub const BITTORRENT: u16 = 37;
    pub const SKYPE_CALL_OUT: u16 = 38;
    pub const MUSICALLY: u16 = 39;
    pub const MEMCACHED: u16 = 40;

    pub const FREE_41: u16 = 41;
    pub const FREE_42: u16 = 42;
    pub const FREE_43: u16 = 43;
    pub const FREE_44: u16 = 44;
    pub const FREE_45: u16 = 45;
    pub const FREE_46: u16 = 46;

    pub const XBOX: u16 = 47;
    pub const QQ: u16 = 48;
    pub const SKYPE_CALL_IN: u16 = 49;
    pub const RTSP: u16 = 50;
    pub const MAIL_IMAPS: u16 = 51;
    pub const ICECAST: u16 = 52;
    pub const PPLIVE: u16 = 53; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const PPSTREAM: u16 = 54;
    pub const ZATTOO: u16 = 55;
    pub const SHOUTCAST: u16 = 56;
    pub const SOPCAST: u16 = 57;
    pub const TVANTS: u16 = 58;
    pub const TVUPLAYER: u16 = 59;
    pub const HTTP_DOWNLOAD: u16 = 60;
    pub const QQLIVE: u16 = 61;
    pub const THUNDER: u16 = 62;
    pub const SOULSEEK: u16 = 63;
    pub const SSL_NO_CERT: u16 = 64;
    pub const IRC: u16 = 65;
    pub const AYIYA: u16 = 66;
    pub const UNENCRYPTED_JABBER: u16 = 67;
    pub const MSN: u16 = 68;
    pub const OSCAR: u16 = 69;
    pub const YAHOO: u16 = 70;
    pub const BATTLEFIELD: u16 = 71;
    pub const GOOGLE_PLUS: u16 = 72;
    pub const IP_VRRP: u16 = 73;
    pub const STEAM: u16 = 74; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const HALFLIFE2: u16 = 75;
    pub const WORLDOFWARCRAFT: u16 = 76;
    pub const TELNET: u16 = 77;
    pub const STUN: u16 = 78;
    pub const IP_IPSEC: u16 = 79;
    pub const IP_GRE: u16 = 80;
    pub const IP_ICMP: u16 = 81;
    pub const IP_IGMP: u16 = 82;
    pub const IP_EGP: u16 = 83;
    pub const IP_SCTP: u16 = 84;
    pub const IP_OSPF: u16 = 85;
    pub const IP_IP_IN_IP: u16 = 86;
    pub const RTP: u16 = 87;
    pub const RDP: u16 = 88;
    pub const VNC: u16 = 89;
    pub const PCANYWHERE: u16 = 90;
    pub const SSL: u16 = 91;
    pub const SSH: u16 = 92;
    pub const USENET: u16 = 93;
    pub const MGCP: u16 = 94;
    pub const IAX: u16 = 95;
    pub const TFTP: u16 = 96;
    pub const AFP: u16 = 97;
    pub const STEALTHNET: u16 = 98;
    pub const AIMINI: u16 = 99;
    pub const SIP: u16 = 100;
    pub const TRUPHONE: u16 = 101;
    pub const DHCPV6: u16 = 103;
    pub const ARMAGETRON: u16 = 104;
    pub const CROSSFIRE: u16 = 105;
    pub const DOFUS: u16 = 106;
    pub const FIESTA: u16 = 107;
    pub const FLORENSIA: u16 = 108;
    pub const GUILDWARS: u16 = 109;
    pub const HTTP_ACTIVESYNC: u16 = 110;
    pub const KERBEROS: u16 = 111;
    pub const LDAP: u16 = 112;
    pub const MAPLESTORY: u16 = 113;
    pub const MSSQL_TDS: u16 = 114;
    pub const PPTP: u16 = 115;
    pub const WARCRAFT3: u16 = 116;
    pub const WORLD_OF_KUNG_FU: u16 = 117;
    pub const SLACK: u16 = 118;
    pub const FACEBOOK: u16 = 119;
    pub const TWITTER: u16 = 120;
    pub const DROPBOX: u16 = 121;
    pub const GMAIL: u16 = 122;
    pub const GOOGLE_MAPS: u16 = 123;
    pub const YOUTUBE: u16 = 124;
    pub const SKYPE: u16 = 125;
    pub const GOOGLE: u16 = 126;
    pub const DCERPC: u16 = 127;
    pub const NETFLOW: u16 = 128;
    pub const SFLOW: u16 = 129;
    pub const HTTP_CONNECT: u16 = 130;
    pub const HTTP_PROXY: u16 = 131;
    pub const CITRIX: u16 = 132; /* It also includes the old NDPI_PROTOCOL_CITRIX_ONLINE */
    pub const LASTFM: u16 = 134;
    pub const WAZE: u16 = 135;
    pub const YOUTUBE_UPLOAD: u16 = 136; /* Upload files to youtube */
    pub const GENERIC: u16 = 137; /* Generic protocol used for category matching */
    pub const CHECKMK: u16 = 138;
    pub const AJP: u16 = 139; /* Leonn Paiva <leonn.paiva@gmail.com> */
    pub const APPLE: u16 = 140;
    pub const WEBEX: u16 = 141;
    pub const WHATSAPP: u16 = 142;
    pub const APPLE_ICLOUD: u16 = 143;
    pub const VIBER: u16 = 144;
    pub const APPLE_ITUNES: u16 = 145;
    pub const RADIUS: u16 = 146;
    pub const WINDOWS_UPDATE: u16 = 147;
    pub const TEAMVIEWER: u16 = 148; /* xplico.org */
    pub const TUENTI: u16 = 149;
    pub const LOTUS_NOTES: u16 = 150;
    pub const SAP: u16 = 151;
    pub const GTP: u16 = 152;
    pub const UPNP: u16 = 153;
    pub const LLMNR: u16 = 154;
    pub const REMOTE_SCAN: u16 = 155;
    pub const SPOTIFY: u16 = 156;
    pub const MESSENGER: u16 = 157;
    pub const H323: u16 = 158; /* Remy Mudingay <mudingay@ill.fr> */
    pub const OPENVPN: u16 = 159; /* Remy Mudingay <mudingay@ill.fr> */
    pub const NOE: u16 = 160; /* Remy Mudingay <mudingay@ill.fr> */
    pub const CISCOVPN: u16 = 161; /* Remy Mudingay <mudingay@ill.fr> */
    pub const TEAMSPEAK: u16 = 162; /* Remy Mudingay <mudingay@ill.fr> */
    pub const TOR: u16 = 163; /* Remy Mudingay <mudingay@ill.fr> */
    pub const SKINNY: u16 = 164; /* Remy Mudingay <mudingay@ill.fr> */
    pub const RTCP: u16 = 165; /* Remy Mudingay <mudingay@ill.fr> */
    pub const RSYNC: u16 = 166; /* Remy Mudingay <mudingay@ill.fr> */
    pub const ORACLE: u16 = 167; /* Remy Mudingay <mudingay@ill.fr> */
    pub const CORBA: u16 = 168; /* Remy Mudingay <mudingay@ill.fr> */
    pub const UBUNTUONE: u16 = 169; /* Remy Mudingay <mudingay@ill.fr> */
    pub const WHOIS_DAS: u16 = 170;
    pub const COLLECTD: u16 = 171;
    pub const SOCKS: u16 = 172; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const NINTENDO: u16 = 173;
    pub const RTMP: u16 = 174; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const FTP_DATA: u16 = 175; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const WIKIPEDIA: u16 = 176; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const ZMQ: u16 = 177;
    pub const AMAZON: u16 = 178; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const EBAY: u16 = 179; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const CNN: u16 = 180; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const MEGACO: u16 = 181; /* Gianluca Costa <g.costa@xplico.org> */
    pub const REDIS: u16 = 182;
    pub const PANDO: u16 = 183; /* Tomasz Bujlow <tomasz@skatnet.dk> */
    pub const VHUA: u16 = 184;
    pub const TELEGRAM: u16 = 185; /* Gianluca Costa <g.costa@xplico.org> */
    pub const VEVO: u16 = 186;
    pub const PANDORA: u16 = 187;
    pub const QUIC: u16 = 188; /* Andrea Buscarinu <andrea.buscarinu@gmail.com> - Michele Campus <michelecampus5@gmail.com> */
    pub const WHATSAPP_VOICE: u16 = 189;
    pub const EAQ: u16 = 190;
    pub const OOKLA: u16 = 191;
    pub const AMQP: u16 = 192;
    pub const KAKAOTALK: u16 = 193; /* KakaoTalk Chat (no voice call) */
    pub const KAKAOTALK_VOICE: u16 = 194; /* KakaoTalk Voice */
    pub const TWITCH: u16 = 195; /* Edoardo Dominici <edoaramis@gmail.com> */
    pub const FREE_196: u16 = 196; /* Free */
    pub const WECHAT: u16 = 197;
    pub const MPEGTS: u16 = 198;
    pub const SNAPCHAT: u16 = 199;
    pub const SINA: u16 = 200;
    pub const HANGOUT: u16 = 201;
    pub const IFLIX: u16 = 202; /* www.vizuamatix.com R&D team & M.Mallawaarachchie <manoj_ws@yahoo.com> */
    pub const GITHUB: u16 = 203;
    pub const BJNP: u16 = 204;
    pub const FREE_205: u16 = 205; /* Free */
    pub const VIDTO: u16 = 206;
    pub const SMPP: u16 = 207; /* Damir Franusic <df@release14.org> */
    pub const DNSCRYPT: u16 = 208;
    pub const TINC: u16 = 209; /* William Guglielmo <william@deselmo.com> */
    pub const DEEZER: u16 = 210;
    pub const INSTAGRAM: u16 = 211; /* Andrea Buscarinu <andrea.buscarinu@gmail.com> */
    pub const MICROSOFT: u16 = 212;
    pub const STARCRAFT: u16 = 213; /* Matteo Bracci <matteobracci1@gmail.com> */
    pub const TEREDO: u16 = 214;
    pub const HOTSPOT_SHIELD: u16 = 215;
    pub const HEP: u16 = 216; /* sipcapture.org QXIP BV */
    pub const GOOGLE_DRIVE: u16 = 217;
    pub const OCS: u16 = 218;
    pub const OFFICE_365: u16 = 219;
    pub const CLOUDFLARE: u16 = 220;
    pub const MS_ONE_DRIVE: u16 = 221;
    pub const MQTT: u16 = 222;
    pub const RX: u16 = 223;
    pub const APPLESTORE: u16 = 224;
    pub const OPENDNS: u16 = 225;
    pub const GIT: u16 = 226;
    pub const DRDA: u16 = 227;
    pub const PLAYSTORE: u16 = 228;
    pub const SOMEIP: u16 = 229;
    pub const FIX: u16 = 230;
    pub const PLAYSTATION: u16 = 231;
    pub const PASTEBIN: u16 = 232;
    pub const LINKEDIN: u16 = 233;
    pub const SOUNDCLOUD: u16 = 234;
    pub const CSGO: u16 = 235;
    pub const LISP: u16 = 236;
    pub const DIAMETER: u16 = 237;
    pub const APPLE_PUSH: u16 = 238;
    pub const GOOGLE_SERVICES: u16 = 239;
    pub const AMAZON_VIDEO: u16 = 240;
    pub const GOOGLE_DOCS: u16 = 241;
    pub const WHATSAPP_FILES: u16 = 242;

    pub fn new() -> Proto {
        Proto {
            master_id: 0,
            app_id: 0,
            category: 0,
        }
    }

    #[inline]
    pub fn success(&self) -> bool {
        self.app_id != Proto::UNKNOWN || self.master_id != Proto::UNKNOWN
    }
}


#[link(name = "layerscpp")]
#[link(name = "ndpi")]
extern "C" {
    pub fn ndpi_detection_process_packet(ctx: *const c_char,
                                         flow: *const c_char,
                                         packet: *const c_char,
                                         packet_len: u16,
                                         tm: u64,
                                         src_id: *const c_char,
                                         dst_id: *const c_char) -> Proto;

    pub fn ndpi_detection_giveup(ctx: *const c_char, flow: *const c_char) -> Proto;

    pub fn ndpi_guess_undetected_protocol(ctx: *const c_char,
                                          proto: u8,
                                          src_ip: u32,
                                          src_port: u16,
                                          dst_ip: u16,
                                          dst_port: u16) -> Proto;



    fn ndpi_protocol2name(ctx: *const c_char,
                          proto: Proto,
                          buf: *mut c_char,
                          len: u32) -> *const c_char;


    pub fn init_ndpi_ctx() -> *const c_char;
    pub fn free_ndpi_ctx(ctx: *const c_char);

    pub fn new_ndpi_flow() -> *const c_char;
    pub fn free_ndpi_flow(ctx: *const c_char);


    pub fn new_ndpi_flow_id() -> *const c_char;
    pub fn free_ndpi_flow_id(ctx: *const c_char);
}


pub struct Detector {
    ctx: *const c_char,
    tcp_dissector_allocator: TCPDissectorAllocator,
}

impl Detector {
    pub fn new(conf: Arc<Configure>) -> Detector {
        let ctx = unsafe { init_ndpi_ctx() };
        Detector {
            ctx,
            tcp_dissector_allocator: TCPDissectorAllocator::new(conf.clone()),
        }
    }


    #[inline]
    pub fn detect(&self,
                  flow: *const c_char,
                  ip_layer: *const c_char,
                  ip_layer_len: u16,
                  tm: u64,
                  src_id: *const c_char,
                  dst_id: *const c_char) -> Proto {
        unsafe {
            ndpi_detection_process_packet(self.ctx, flow, ip_layer as *const c_char, ip_layer_len, tm, src_id, dst_id)
        }
    }

    pub fn protocol_name(&self, proto: &Proto) -> String {
        let mut array: [u8; 16] = [0; 16];
        let c_str;
        unsafe {
            ndpi_protocol2name(self.ctx, *proto, array.as_mut_ptr() as *mut i8, 16);
            c_str = CStr::from_bytes_with_nul_unchecked(&array);
        }
        return c_str.to_string_lossy().into_owned();
    }

    pub fn alloc_tcp_dissector(&self, proto: &Proto) -> Rc<RefCell<TCPDissector>> {
        self.tcp_dissector_allocator.alloc_dissector(proto)
    }
}


impl Drop for Detector {
    fn drop(&mut self) {
        debug!("detector cleanup");
        unsafe {
            free_ndpi_ctx(self.ctx);
        }
    }
}