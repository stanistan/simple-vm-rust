const simple_vm = import('./simple_vm_wasm');
simple_vm.then(vm => {
    console.log('attaching to window as simple_vm');
    window.simple_vm = vm;
});
