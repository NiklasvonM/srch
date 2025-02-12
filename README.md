# srch

Search JSONs for values based on the path.

## Example Usage

`srch "fieldOne.isPresent: true" example_files/*.json`

examples_files/test.json:

```json
{
    "someList": [
        {
            "fieldOne": {
                "isPresent": false
            },
            "fieldTwo": {
                "isPresent": false
            }
        },
        {
            "fieldOne": {
                "isPresent": true
            },
            "fieldTwo": {
                "isPresent": true
            }
        }
    ]
}
```
Output: `someList.1.fieldOne.isPresent`

| Command                                                     | Output                                                         |
| ----------------------------------------------------------- | -------------------------------------------------------------- |
| srch "fieldOne.isPresent: true" example_files/*.json        | someList.1.fieldOne.isPresent                                  |
| srch '1.fieldOne.isPresent: true' example_files/test.json`  | someList.1.fieldOne.isPresent                                  |
| srch '0.fieldOne.isPresent: true' example_files/test.json`  |                                                                |
| srch "isPresent: true" example_files/*.json                 | someList.1.fieldOne.isPresent<br>someList.1.fieldTwo.isPresent |
| srch "isPresent: true" example_files/*.json -s              | someList.1.fieldOne.isPresent                                  |
| srch "isPresent: true" example_files/*.json -p              | example_files/test.json<br>example_files/test.json             |
| srch "isPresent: true" example_files/*.json -p              | example_files/test.json<br>example_files/test.json             |
| srch "isPresent: true" example_files/*.json -s -p           | example_files/test.json                                        |
| cat example_files/test.json \| srch "isPresent: true"       | someList.1.fieldOne.isPresent<br>someList.1.fieldTwo.isPresent |
| cat example_files/test.json \| srch "isPresent: true" \| wc | 2       2      60                                              |

## Search Term Syntax

The search path and value that is sought are separated by a colon ":".
The field names in the field path is separated by dots ".". Integers are interpreted as list indices, starting at 0.
