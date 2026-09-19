import 'package:flutter_test/flutter_test.dart';

import 'package:client_flutter/main.dart';

void main() {
  testWidgets('shows the engineering workbench', (WidgetTester tester) async {
    await tester.pumpWidget(const CivilEngineeringApp(kernelReady: true));

    // Arabic is the default locale; English remains available as a fallback.
    expect(find.text('منصة الهندسة المدنية'), findsWidgets);
    expect(find.text('مساحة العمل الهندسية'), findsOneWidget);
    expect(find.text('النواة متصلة'), findsOneWidget);
  });
}
