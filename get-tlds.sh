#!/usr/bin/bash

curl https://publicsuffix.org/list/public_suffix_list.dat | grep -P "^[^/]|^// =" > src/public_suffix_list.dat
