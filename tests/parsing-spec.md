# Parsing logical units

|PARSE-SPECS.1|
: We can parse a file of logical units into different formats, preserving all
  critical content of the logical unit content.

## Serialization Format

Supported formats include:

|PARSE-SPECS.1::FORMAT.1|
: Must support parsing specs into machine readable formats.

|PARSE-SPECS.1::FORMAT.1::JSON.1|
: Must support parsing a file of specs into JSON.

|PARSE-SPECS.1::FORMAT.1::CSV.1|
: Must support parse a file of specs into CSV.

## Content

|PARSE-SPECS.1::CONTENT.1|
: Parsing must support all expected forms of content.

|PARSE-SPECS.1::CONTENT.1::MULTI-PARA.1|
: The content of logical units must be preserved.
: Even when it spans multiple paragraphs.
: - Or
 - includes
 - lists

|PARSE-SPECS.1::CONTENT.1::INLINE.1|
: The folowing inline styling must be preserved:
: - **Strong** (__both__ ways)
 - *Emphasizes* (_both_ ways)
 - ~~Strikethrough~~
 - `code`
 - [links](/url)
 - ![images](/url)
 - [smallcaps]{.smallcaps}
