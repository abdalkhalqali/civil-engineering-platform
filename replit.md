# Civil Engineering Platform

## تشغيل المعاينة

شغّل Workflow `Flutter Web Preview` من Replit، أو نفّذ من جذر المشروع:

```bash
./scripts/start_flutter_web.sh
```

يُبنى تطبيق Flutter Web على المنفذ `5000`. السكربت يجهّز Flutter 3.47.4،
يبني نواة Rust إلى WebAssembly، ثم يضيف ترويسات العزل المطلوبة لـ
`flutter_rust_bridge`.

## مساحة العمل ثلاثية الأبعاد

تبدأ الواجهة من `apps/client_flutter/lib/screens/workbench_screen.dart`.
المشهد الهندسي التفاعلي موجود في
`apps/client_flutter/lib/widgets/engineering_viewport.dart`، وهو رسم ثلاثي
الأبعاد فعلي عبر إسقاط نقاط العالم إلى Canvas، وليس صورة ثابتة أو Mockup.

يدعم الـViewport حاليًا:

- شبكة هندسية ومحاور X/Y/Z.
- Orbit بالسحب، وPan من خلال وضع التحريك، وZoom بعجلة الماوس.
- Perspective وOrthographic.
- مجسمًا إنشائيًا أوليًا من أعمدة وكمرات وبلاطة.
- تحديد الأعمدة لتهيئة مستكشف العناصر والخصائص لاحقًا.
- أداة إنشاء الأعمدة بالنقر على نقطة في المشهد، وأداة رسم الكمرة بين نقطتين؛
  كلاهما يحدّث النموذج في Rust ثم يعيد اشتقاق المشهد.
- مرشحات إظهار وإخفاء فئات العناصر، ووحدة عرض قابلة للتغيير.
- على الهاتف يظهر زر «لوحة الخصائص» لفتح مستكشف المشروع من الجانب بدل فقدانه.

تظهر حالة اتصال Rust في رأس الشاشة بعد نجاح `RustLib.init()`.

## التحقق

```bash
cd apps/client_flutter
flutter analyze
flutter test
```

لاختبارات الجسر على جهاز التطوير، يجب أولًا بناء المكتبة محليًا ثم تمرير مسارها:

```bash
cd core/geometry_kernel_rs && cargo build
cd ../../apps/client_flutter
FRB_DART_LOAD_EXTERNAL_LIBRARY_NATIVE_LIB_DIR=../../core/geometry_kernel_rs/target/debug flutter test
```