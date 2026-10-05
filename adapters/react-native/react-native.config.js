// Autolinking for the native module + the @rva/kotlin Android library.
module.exports = {
  dependency: {
    platforms: {
      ios: {},
      android: {
        // The Android module depends on the @rva/kotlin AAR (librva_ffi.so + JNI).
        packageImportPath: "import dev.rva.rn.RNRvaPackage;",
        packageInstance: "new RNRvaPackage()",
      },
    },
  },
};
