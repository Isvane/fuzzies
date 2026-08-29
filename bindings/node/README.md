## Installation

```bash
npm install fuzzies-node
# or yarn add / bun add
```

## Example

```javascript
const { Dictionary } = require('fuzzies-node');

function main() {
  // Prepare your raw text file (must be sorted lexicographically)
  // Fuzzies provides a handy in-place sorter for convenience:
  Dictionary.sort('words.txt');

  // Build the immutable binary FST from the sorted text file
  Dictionary.build('words.txt', 'words.fst');

  // Load the dictionary (memory-mapped from disk)
  const dict = Dictionary.open('words.fst');

  // Alternatively, build directly from an array in memory:
  // const dict = Dictionary.fromArray(["apple", "banana", "cherry"]);

  // Check for exact matches instantly
  if (dict.contains('banana')) {
    console.log('Exact match found!');
  }

  // Quick single-suggestion lookup
  const topMatch = dict.suggest('banaan');
  if (topMatch) {
    console.log(`Best suggestion: ${topMatch.key}`);
  }

  // Perform a fuzzy search with custom options
  const results = dict.search('banaan', {
    distance: 2,
    transposition: true, // Handles adjacent swaps (e.g., "teh" -> "the")
    prefix: false,       // Set to true for prefix fuzzy lookups
    limit: 5
  });

  for (const result of results) {
    console.log(`Found: ${result.key} (distance: ${result.distance})`);
  }
}
```

## Missing Features
- No `batch_search()`: The multithreaded Rayon batch search from the Rust crate is not yet exposed.
- Range Bounds: The `[ge, gt, le, lt]` options for restricting search bounds are currently unavailable in the Node binding.
- Buffer Loading: You cannot yet initialize a dictionary directly from a Node.js `Buffer` in memory (only via array or file path).

## License

This project is licensed under the [MIT license.](LICENSE)
