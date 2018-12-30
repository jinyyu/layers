layers
=======


build
-----

nDPI

    git clone git@github.com:ntop/nDPI.git
    cd nDPI
    git checkout 2.6-stable
    ./autogen.sh
    ./configure
    sudo make install

gmime


    git clone git@github.com:GNOME/gmime.git
    cd gmime
    git checkout 3.2.3
    ./autogen.sh
    ./configure
    sudo make install


c++ module

    mkdir -p build
    cd build
    cmake3 ..
    sudo make install


rust

    cargo build

run
---

    vim /etc/layers/config.yaml
    sudo RUST_LOG=debug ./target/debug/layers
   


