import React from "react";
import { Platform, SafeAreaView, StyleSheet, Text, View } from "react-native";
import { RVAImage } from "@airovo/rva-react-native";

// The .rva asset and the @airovo/rva-web runtime are fetched by the WebView, so both
// need to be reachable from the device. 10.0.2.2 is the Android emulator's
// alias for the host machine; iOS simulators can use localhost.
const HOST = Platform.select({ android: "10.0.2.2", default: "localhost" });
const ASSET = `http://${HOST}:8080/hero.rva`;
const BASE = `http://${HOST}:8080`;

export default function App() {
  return (
    <SafeAreaView style={styles.safe}>
      <View style={styles.header}>
        <Text style={styles.title}>RVA — one asset, any shape</Text>
        <Text style={styles.subtitle}>
          WebView-based React Native adapter over the WASM core
        </Text>
      </View>
      <RVAImage
        src={ASSET}
        baseUrl={BASE}
        alt="Responsive campaign hero"
        style={styles.hero}
      />
    </SafeAreaView>
  );
}

const styles = StyleSheet.create({
  safe: { flex: 1, backgroundColor: "#f8fafc" },
  header: { paddingHorizontal: 16, paddingTop: 16, gap: 4 },
  title: { fontSize: 18, fontWeight: "700", color: "#0f172a" },
  subtitle: { color: "#475569" },
  hero: { flex: 1, margin: 16, borderRadius: 16 },
});
