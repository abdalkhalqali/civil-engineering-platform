// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Arabic (`ar`).
class AppLocalizationsAr extends AppLocalizations {
  AppLocalizationsAr([String locale = 'ar']) : super(locale);

  @override
  String get appTitle => 'منصة الهندسة المدنية';

  @override
  String get screenTitle => 'منصة الهندسة المدنية';

  @override
  String get bridgeDescription =>
      'واجهة إثبات اتصال بين Flutter ونواة Rust الهندسية';

  @override
  String get testRustKernel => 'اختبار نواة Rust';

  @override
  String get kernelConnected => 'تم الاتصال بنواة الهندسة بنجاح.';

  @override
  String bridgeError(String error) {
    return 'خطأ في الجسر: $error';
  }
}
