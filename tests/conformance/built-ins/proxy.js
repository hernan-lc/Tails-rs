/*---
description: Tests Proxy creation and standard traps (get, set, has).
---*/

// Basic creation and target forwarding
const target = { x: 1, y: 2 };
const handler = {};
const proxy = new Proxy(target, handler);
assert.sameValue(proxy.x, 1, "Proxy get forwarding");
assert.sameValue(proxy.y, 2, "Proxy get forwarding");

// Proxy get trap
const targetGet = { x: 1 };
const handlerGet = {
    get: function(t, prop, receiver) {
        if (prop === "x") {
            return 42;
        }
        return t[prop];
    }
};
const proxyGet = new Proxy(targetGet, handlerGet);
assert.sameValue(proxyGet.x, 42, "Proxy get trap interception");

// Proxy set trap
const targetSet = { x: 1 };
const handlerSet = {
    set: function(t, prop, value, receiver) {
        t[prop] = value * 2;
        return true;
    }
};
const proxySet = new Proxy(targetSet, handlerSet);
proxySet.x = 5;
assert.sameValue(targetSet.x, 10, "Proxy set trap interception");

// Proxy has trap
const targetHas = { x: 1 };
const handlerHas = {
    has: function(t, prop) {
        if (prop === "secret") {
            return false;
        }
        return prop in t;
    }
};
const proxyHas = new Proxy(targetHas, handlerHas);
assert.sameValue("x" in proxyHas, true, "Proxy has trap true");
assert.sameValue("secret" in proxyHas, false, "Proxy has trap false");
assert.sameValue("secret" in targetHas, false, "Target has false");
targetHas.secret = 42;
assert.sameValue("secret" in targetHas, true, "Target has true");
assert.sameValue("secret" in proxyHas, false, "Proxy has trap false despite target having property");
