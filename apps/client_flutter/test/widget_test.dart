import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:client_flutter/main.dart';

void main() {
  testWidgets('shows the Rust kernel bridge proof of concept screen',
      (WidgetTester tester) async {
    await tester.pumpWidget(const CivilEngineeringApp());

    // Arabic is the default locale; English remains available as a fallback.
    expect(find.text('منصة الهندسة المدنية'), findsWidgets);
    expect(
      find.widgetWithText(FilledButton, 'اختبار نواة Rust'),
      findsOneWidget,
    );
  });
}
