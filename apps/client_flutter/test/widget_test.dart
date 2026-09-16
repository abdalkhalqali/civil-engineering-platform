import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:client_flutter/main.dart';

void main() {
  testWidgets('shows the Rust kernel bridge proof of concept screen',
      (WidgetTester tester) async {
    await tester.pumpWidget(const CivilEngineeringApp());

    // Title is shown in the app bar and as the screen heading.
    expect(find.text('Civil Engineering Platform'), findsWidgets);
    expect(find.widgetWithText(FilledButton, 'Test Rust Kernel'), findsOneWidget);
  });
}
