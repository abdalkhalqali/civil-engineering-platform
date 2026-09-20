import 'package:flutter_test/flutter_test.dart';

import 'package:client_flutter/ffi_bridge/generated/api.dart';
import 'package:client_flutter/ffi_bridge/generated/frb_generated.dart';

/// End to end checks of the Rust bridge:
/// Dart -> flutter_rust_bridge -> Rust kernel -> value -> Dart.
///
/// A real device/app build bundles the native library through cargokit. When the
/// test runs on a desktop host against a locally built library, point
/// `FRB_DART_LOAD_EXTERNAL_LIBRARY_NATIVE_LIB_DIR` at the directory that holds
/// `libgeometry_kernel_rs.so` (for example `core/geometry_kernel_rs/target/debug`).
void main() {
  setUpAll(() async {
    await RustLib.init();
  });

  test('get_kernel_status() returns the Rust kernel status string', () {
    expect(
      getKernelStatus(),
      'Engineering Geometry Kernel (Rust) is connected successfully!',
    );
  });

  test('create_empty_model() reaches the engineering model core in Rust', () {
    final summary = createEmptyModel();

    expect(summary.schemaVersion, 1);
    expect(summary.revision, BigInt.zero);
    expect(summary.projectId.length, 36);
    expect(summary.levels, 0);
    expect(summary.grids, 0);
    expect(summary.materials, 0);
    expect(summary.crossSections, 0);
    expect(summary.elements, 0);
  });

  test('two empty models are two distinct projects', () {
    expect(createEmptyModel().projectId, isNot(createEmptyModel().projectId));
  });
}
