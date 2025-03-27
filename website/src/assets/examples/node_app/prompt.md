Project Path: node_app

Source Tree:

```txt
node_app
├── README.md
├── data
│   └── sample.json
└── src
    ├── index.js
    └── utils.js
```

`node_app/README.md`:

```md
Simple Node.js app to process JSON data.
Feature idea: Add a filter function to sort users by age.
```

`node_app/data/sample.json`:

```json
{
    "users": [
        {
            "name": "Alice",
            "age": 25
        },
        {
            "name": "Bob",
            "age": 30
        }
    ]
}
```

`node_app/src/index.js`:

```js
const { processData } = require("./utils");

console.log("App started");
processData();
```

`node_app/src/utils.js`:

```js
function processData() {
  console.log("Processing data...");
}

module.exports = { processData };
```
