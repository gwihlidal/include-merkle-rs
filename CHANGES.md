# Changes

## 0.1.4 (2019-01-29)

* Make normalize line endings a parameter, to stay un-opinionated.
* More robust source file loading to handle different character sets / encodings.

## 0.1.3 (2019-01-29)

* Normalize line endings to Unix LF.

## 0.1.2 (2018-12-23)

* Fixed sorting bug.
* Added `get_root_node` convenience function.

## 0.1.1 (2018-12-23)

* Added some documentation.
* Reverse sort pattern matches by range start to ensure safe patching (need to maintain correctly offsets into the original source).
* Minor improvements and added `path_strip_base` convenience function.

## 0.1.0 (2018-12-23)

* First release.