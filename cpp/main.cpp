#include <map>
#include <stdio.h>

int main()
{
    std::map<int, const char*> map;
    map[1] = "one";
    map[2] = "two";
    map[3] = "three";

    for (auto it = map.begin(); it != map.end();) {
        if (it->first % 2 == 0) {
            it = map.erase(it);
        }
        else {
            ++it;
        }
    }


    for (auto it = map.begin(); it != map.end(); ++it) {
        printf("%d-%s\n", it->first, it->second);
    }
}