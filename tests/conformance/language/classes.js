/*---
description: Tests class declarations, constructors, methods, and inheritance.
---*/

// Basic class
class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
    toString() {
        return "Point(" + this.x + "," + this.y + ")";
    }
}
let p = new Point(3, 4);
assert.sameValue(p.x, 3, "class constructor field assign");
assert.sameValue(p.y, 4, "class constructor field assign");
assert.sameValue(p.toString(), "Point(3,4)", "class instance method");

// Inheritance
class ColorPoint extends Point {
    constructor(x, y, color) {
        super(x, y);
        this.color = color;
    }
    getColor() {
        return this.color;
    }
}
let cp = new ColorPoint(5, 10, "red");
assert.sameValue(cp.x, 5, "inheritance fields");
assert.sameValue(cp.color, "red", "child class field assign");
assert.sameValue(cp.toString(), "Point(5,10)", "inherited method call");
assert.sameValue(cp.getColor(), "red", "child method call");
