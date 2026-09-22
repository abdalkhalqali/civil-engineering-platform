# Civil Engineering Platform

## المبدأ

```
User Input → User Intent → Engineering Command → Engineering Model → Render Data → Viewport
```

**`EngineeringModel` في Rust هو مصدر الحقيقة الوحيد.** لا يوجد نموذج هندسي ثانٍ في
Dart أو في JavaScript: الواجهة ترسل *أوامر* وتستقبل *بيانات عرض مشتقّة* فقط.

```text
core/geometry_kernel_rs      ← النواة: النموذج والأوامر والتحقق والحفظ
  model/                     ← EngineeringModel (مستويات، شبكات، مواد، قطاعات، عناصر)
  elements/                  ← Column, Beam, Slab, Wall, Foundation
  commands/                  ← ModelCommand + ElementCommand + StateOp (تراجع/إعادة)
  history.rs                 ← تراجع وإعادة على حالة النموذج
  session.rs                 ← ModelSession: طبقة التطبيق فوق النواة
  render/                    ← RenderData: إسقاط مشتق للنموذج
  api.rs                     ← حد Flutter (flutter_rust_bridge)
  wasm.rs                    ← حد المتصفح (wasm-bindgen) — نفس ModelSession

apps/client_flutter          ← عميل Flutter (الهاتف/اللوحي/القلم)
web/                         ← بيئة العمل في المتصفح (Renderer + إدخال فقط)
```

## تشغيل بيئة العمل في المتصفح

النواة تُترجم إلى WebAssembly، والصفحة تدفعها مباشرة:

```bash
sh ./scripts/build_wasm.sh   # يبني النواة إلى web/pkg (يحتاج Rust + wasm-bindgen 0.2.128)
npm run dev                  # يقدّم web/ على المنفذ 3000
```

`web/app.js` هو **مُصيّر وجهاز إدخال** فقط: لا يحسب هندسة، ولا يخزّن خصائص عنصر،
ولا يقرر ما هو العمود. كل رقم يُعرض يأتي من النواة.

## تشغيل عميل Flutter

```bash
./scripts/start_flutter_web.sh   # Flutter 3.47.4 + Rust → WASM + خادم بترويسات العزل
```

## التحقق

```bash
cd core/geometry_kernel_rs
cargo test                  # 85 اختبارًا: النموذج، الأوامر، التراجع، العرض، الحفظ
cargo check --target wasm32-unknown-unknown

cd apps/client_flutter
flutter analyze && flutter test
```

توليد الجسر (لا يُعدَّل يدويًا أبدًا):

```bash
cd apps/client_flutter
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
```

## ما ينجزه الشريحة الرأسية الحالية

1. **مشروع جديد** → مستويان (0.00 م و 3.20 م) + شبكة من أربعة محاور (A, B, 1, 2)
   + مادة خرسانة C30 + قطاع 400×400 — كلها كائنات في النموذج الهندسي.
2. **إنشاء عمود** على تقاطع شبكة باللمس أو القلم، مع التقاط مرئي (A-1) وإدخال بالمتر.
3. **تحديد العنصر** وعرض خصائصه من النموذج.
4. **المقبض العلوي ⇕**: سحب رأس العمود وإفلاته على مستوى آخر يرسل
   `SetElementTopLevel` — فيتغيّر **مرجع المستوى في النموذج**، وتتبع الهندسة المشتقّة
   هذا التغيير. لا يُرفع مجسم في الشاشة.
5. **تعديل القطاع** 400×400 → 500×500 مم: يُعاد استخدام قطاع مشترك أو يُنشأ،
   والمساحة المشتقّة تتغير معه.
6. **حذف** العنصر من النموذج (لا إخفاء).
7. **تراجع/إعادة** على حالة النموذج (وليس على واجهة العرض).
8. **حفظ `.civilx`** ثم فتحه: النموذج يُعاد بناؤه من الملف، والهندسة والمشهد تُشتقّ منه.
9. **تحريك الكاميرا لا يغيّر `revision`** ولا بايتًا واحدًا من المشروع.

## ما لم يُنفَّذ بعد (مقصود)

- علاقات بين العناصر (كمرة ↔ عمود، أساس ↔ عمود) كمعرّفات محفوظة.
- قيود هندسية (توازي، تعامد، محاذاة) — والأساس جاهز لها في `Transform3D`.
- مقابض السحب الحر على الشاشة (Move/Extend/Rotate/Mirror/Offset/Array).
- الإدخال الرقمي أثناء السحب (Dynamic Input) كتحرير مباشر.
- رسم البلاطة بحدود من عدة نقاط (Sketch Mode)، والفتحات.
- الأسقف والسقف المائل والسلالم.
- المساقط والقطاعات والأبعاد (2D مشتق من نفس النموذج).
- تحديثات تدريجية (`RenderData` تُعاد كاملة الآن).
- حصة الكميات، التحليل الإنشائي، IFC/BIM، المساعد الذكي.
