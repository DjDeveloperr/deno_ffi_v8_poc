const lib = Deno.dlopen("./add.dll", {
  add: {
    parameters: ["i32", "i32"],
    result: "i32",
  },
});

let min, max, sum = 0, iters = 1_000_000;

for (let i = 0; i < iters; i++) {
  const before = performance.now();
  lib.symbols.add(1, 2);
  const after = performance.now();
  const diff = after - before;
  if (i === 0) {
    min = diff;
    max = diff;
  } else {
    min = Math.min(min, diff);
    max = Math.max(max, diff);
  }
  sum += diff;
}

console.log(
  `${iters} iters, ${((sum / iters) * 1000000).toFixed(2)} ns/iter, ${
    (min * 1000000).toFixed(2)
  } ns min, ${(max * 1000000).toFixed(2)} ns max`,
);
