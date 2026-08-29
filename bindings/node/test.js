import { Dictionary } from ".";

const dict = Dictionary.open('../../dict.fst');


console.log(dict.search("appple", { distance: 1, limit: 5 }));
console.log(dict.search("gleam", { distance: 1, limit: 5, prefix: true }));
