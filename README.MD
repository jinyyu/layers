layers
=======

nDPI
____

    git clone git@github.com:ntop/nDPI.git
    cd nDPI
    git checkout 2.6-stable
    ./autogen.sh
    ./configure
    sudo make install

gmime
_____

    git clone git@github.com:GNOME/gmime.git
    cd gmime
    git checkout 3.2.3
    ./autogen.sh
    ./configure
    sudo make install


c++ module
__________

    mkdir -p build
    cd build
    cmake3 ..
    sudo make install


rust
____

    cargo build

run
---
   vim /etc/layers/config.yaml


