# client_flutter

واجهة Flutter العربية لمنصة الهندسة المدنية.

## الحالة الحالية

هذه المرحلة تؤسس واجهة عربية أولية باتجاه RTL مع دعم إنجليزي احتياطي. الواجهة الحالية
ما تزال شاشة تحقق من اتصال Flutter بنواة Rust؛ لا تحتوي على إدارة مشاريع أو محررات
هندسية بعد.

ملفات الترجمة موجودة في `lib/l10n/` بصيغة ARB، ويولد Flutter ملف
`app_localizations.dart` عند تشغيل توليد الترجمة.

## الإصدارات المرجعية

- Flutter 3.47.4
- Dart 3.13.3
- flutter_rust_bridge 2.13.0

## التحقق

```bash
flutter pub get
flutter analyze
flutter test
```

في البيئات التي لا تحتوي على Flutter أو Dart، تسجل فحوصات Flutter كـ
`NOT TESTED — ENVIRONMENT LIMITATION` ولا تعتبر فشلًا في Rust Core.

## تشغيل المعاينة في Replit

من جذر المستودع:

```bash
./scripts/start_flutter_web.sh
```

السكربت يستخدم Flutter 3.47.4 ويشغل Flutter Web على `0.0.0.0:5000`، وهو المنفذ
المطلوب للمعاينة عبر Replit.