# `theca` dev tools

this folder contains various (well one right now) tools that are somewhat useful
for the developement of theca.

## tools

### profile patching template

`profile_patcher_template.py` is a template for creating patches for *theca* profile
files, including functions to decrypt/encrypt theca profiles using properly derived
AES keys.

this was originally written in order to convert encrypted profiles from one encryption
format to another. it could also be used, for instance, to convert note timeformats or
something idk.
