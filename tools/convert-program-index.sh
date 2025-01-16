#!/usr/bin/env bash

excluded_packages="'busybox','fltk','fltk14'"

sql_filter="system = '$2-linux'"
sql_filter+=" AND package NOT IN ($excluded_packages)"
sql_filter+=" AND name NOT LIKE 'nix%'"

sqlite3 -csv "$1" "SELECT name, package FROM Programs
WHERE $sql_filter
GROUP by name, package
ORDER by name, package;"
