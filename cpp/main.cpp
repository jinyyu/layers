#include <ndpi/ndpi_api.h>
#include <stdio.h>


int main(int argc, char* argv[]){
    fprintf(stderr, "%d\n", sizeof(ndpi_protocol));
    NDPI_PROTOCOL_TFTP
}
