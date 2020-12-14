import('./pkg/index').catch(console.error);

// I don't believe it's possible to eval inside a specific scope besides the global scope. The es5 spec says
// that if we evaluate eval indirectly it will run in the global scope, so use this indirect reference
// to achieve that.
// https://www.ecma-international.org/ecma-262/5.1/#sec-10.4.2
window.globalEval = eval;