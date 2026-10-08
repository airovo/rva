import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:rva_flutter/rva_flutter.dart';

void main() => runApp(const ExampleApp());

class ExampleApp extends StatelessWidget {
  const ExampleApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(
      title: 'RVA Flutter example',
      home: HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  Uint8List? _bytes;
  String _status = 'Loading…';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final data = await rootBundle.load('assets/hero.rva');
    setState(() {
      _bytes = data.buffer.asUint8List();
      _status = 'hero.rva — resize the window to recompose';
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('RVA Flutter example')),
      body: _bytes == null
          ? Center(child: Text(_status))
          : Column(
              children: [
                Expanded(child: RVAImageView(bytes: _bytes!)),
                Padding(
                  padding: const EdgeInsets.all(8),
                  child: Text(_status, style: Theme.of(context).textTheme.bodySmall),
                ),
              ],
            ),
    );
  }
}
