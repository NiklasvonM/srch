# srch

Search JSONs for data based on the path.

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

Piping `cat` into `srch`:
`cat example_files/test.json | srch "fieldOne.isPresent: true"`

Piping into `wc`: `srch "fieldOne.isPresent: true" example_files/*.json | wc`
