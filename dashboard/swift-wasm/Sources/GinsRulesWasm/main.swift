import JavaScriptKit

let document = JSObject.global.document

func version() -> JSValue {
  return "Gins-Rules SwiftWasm 1.0.0".jsValue()
}

let ginsRules = JSObject.global.Object.function!.new()
ginsRules.version = .function { _ in version() }

JSObject.global.ginsRules = .object(ginsRules)

print("Gins-Rules SwiftWasm Initialized")
