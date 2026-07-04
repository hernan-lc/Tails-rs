let proto = { inherited: 99 };
let obj = Object.create(proto);
obj.own = 1;
obj.inherited + obj.own;
console.log(obj);
let obj2 = { value: [1, 2, 3] };
// Force GC by allocating
for (let i = 0; i < 1000; i++) {
  Array(1);
}
obj2.value[0];
console.log(obj2);
