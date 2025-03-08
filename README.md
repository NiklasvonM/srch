# srch

Search JSONs for values based on the path.

## Example Usage

`srch fieldOne.index 2 example_files/*.json`

Output: `someList.1.fieldOne.index: 2`

examples_files/test.json:
```json
{
    "someList": [
        {
            "fieldOne": {
                "irrelevantInformation": 0,
                "isPresent": false,
                "index": 0
            },
            "fieldTwo": {
                "irrelevantInformation": 0,
                "isPresent": false,
                "index": 1
            }
        },
        {
            "fieldOne": {
                "isPresent": true,
                "index": 2
            },
            "fieldTwo": {
                "isPresent": true,
                "index": 3
            }
        }
    ]
}
```

| Command                                                  | Output                                                                                       |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| srch index "1\|2" example_files/*.json                   | someList.0.fieldTwo.index: 1<br>someList.1.fieldOne.index: 2                                 |
| srch index "[0-2]" example_files/*.json                  | someList.0.fieldOne.index: 0<br>someList.0.fieldTwo.index: 1<br>someList.1.fieldOne.index: 2 |
| srch index "[^1]" example_files/*.json                   | someList.0.fieldOne.index: 0<br>someList.1.fieldOne.index: 2<br>someList.1.fieldTwo.index: 3 |
| srch index ">1<4" example_files/*.json -n                 | someList.1.fieldOne.index: 2<br>someList.1.fieldTwo.index: 3                                 |
| srch fieldOne.isPresent true example_files/*.json        | someList.1.fieldOne.isPresent: true                                                          |
| srch 1.fieldOne.isPresent true example_files/test.json   | someList.1.fieldOne.isPresent: true                                                          |
| srch 0.fieldOne.isPresent true example_files/test.json   |                                                                                              |
| srch isPresent true example_files/*.json                 | someList.1.fieldOne.isPresent: true<br>someList.1.fieldTwo.isPresent: true                   |
| srch isPresent true example_files/*.json -s              | someList.1.fieldOne.isPresent: true                                                          |
| srch isPresent true example_files/*.json -p              | example_files/test.json<br>example_files/test.json                                           |
| srch isPresent true example_files/*.json -s -p           | example_files/test.json                                                                      |
| cat example_files/test.json \| srch isPresent true       | someList.1.fieldOne.isPresent: true<br>someList.1.fieldTwo.isPresent: true                   |
| cat example_files/test.json \| srch isPresent true \| wc | 2       4      72                                                                            |

## Search Term Syntax

The field names in the field path are separated by dots "." by default, can be changed via `-f` flag. Integers are interpreted as list indices, starting at 0. Only the "tail" of the field path needs to be specified.

The search term is interpreted as a regular expression, unless the `-n`/`--numeric` flag is used. Then up to two of the operations "<=", ">=", "<" and ">" are allowed, for examples ">0<2" or "<100".

## Indepth Examples

### Finding Files With Multiple Conditions

To find all files that have fieldOne.isPresent false _and_ fieldTwo.isPresent true, you can use `srch` with the path `-p` and single `-s` flag together with `uniq -d` and process substitution:

`sort <(srch fieldOne.isPresent false example_files/*.json -p -s) <(srch fieldTwo.isPresent true example_files/*.json -p -s) | uniq -d`

To find all files where (at least) one of the above conditions is met, remove the `-d` flag:

`sort <(srch fieldOne.isPresent false example_files/*.json -p -s) <(srch fieldTwo.isPresent true example_files/*.json -p -s) | uniq`

### Calculating the Sum of Values

`srch index "[0-9]" example_files/test.json | awk -F ': ' '{sum += $2} END {print sum}'`

### Exclude Data From Specific Field

`srch isPresent true example_files/*.json | grep -v "fieldTwo"`

### Copying Files Conditionally

`srch isPresent true example_files/*.json -p -s | xargs -I {} cp {} backup_dir/`

### Tabular Output

`srch index "[0-3]" example_files/*.json | column -t -s ':'`

### Adding Line Numbers

`srch index "[0-3]" example_files/*.json | nl`

### Conditional Logic in Scripts

```bash
#!/bin/bash

output=$(srch isPresent true example_files/*.json)

if [ -z "$output" ]; then
  echo "Did not find 'isPresent: true'."
else
  echo "Found 'isPresent: true' in at least one file."
fi
```
