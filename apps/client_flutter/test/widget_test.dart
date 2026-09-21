import 'package:flutter/material.dart';
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

  testWidgets('fits a phone viewport without layout exceptions', (
    WidgetTester tester,
  ) async {
    tester.view.physicalSize = const Size(390, 844);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(const CivilEngineeringApp(kernelReady: true));

    expect(find.text('مساحة العمل الهندسية'), findsNothing);
    expect(find.byIcon(Icons.create_new_folder_outlined), findsOneWidget);
    expect(find.text('لوحة الخصائص'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('opens the new project form', (WidgetTester tester) async {
    await tester.pumpWidget(const CivilEngineeringApp(kernelReady: true));
    await tester.tap(find.byIcon(Icons.create_new_folder_outlined));
    await tester.pumpAndSettle();

    expect(find.text('إنشاء مشروع جديد'), findsOneWidget);
    expect(find.text('نوع المشروع'), findsOneWidget);
    expect(find.text('المساحة التقديرية لقطعة الأرض'), findsOneWidget);
  });
}
