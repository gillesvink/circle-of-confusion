package org.codeberg.gillesvink;

import com.dylibso.chicory.runtime.ExportFunction;
import com.dylibso.chicory.wasm.types.Value;
import com.dylibso.chicory.wasm.Parser;
import com.dylibso.chicory.wasm.WasmModule;
import com.dylibso.chicory.runtime.Instance;
import java.io.File;

public class CircleOfConfusion {
    public static void main(String[] args) {
        // point this to your path on disk
        WasmModule module = Parser
                .parse(new File("/home/gillesvink/Developer/circle-of-confusion/target/wasm32-unknown-unknown/release/circle_of_confusion.wasm"));
        Instance instance = Instance.builder(module).build();

        System.out.println(String.format("Hello World! %s", instance.functionCount()));
    }
}
