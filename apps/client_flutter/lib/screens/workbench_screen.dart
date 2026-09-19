import 'package:flutter/material.dart';

import '../ffi_bridge/generated/api.dart';
import '../widgets/engineering_viewport.dart';

class WorkbenchScreen extends StatefulWidget {
  const WorkbenchScreen({required this.kernelReady, super.key});

  final bool kernelReady;

  @override
  State<WorkbenchScreen> createState() => _WorkbenchScreenState();
}

class _WorkbenchScreenState extends State<WorkbenchScreen> {
  ViewportInteraction _interaction = ViewportInteraction.orbit;
  ViewportProjection _projection = ViewportProjection.perspective;
  String? _selectedElement;

  void _checkKernel() {
    try {
      getKernelStatus();
      _showMessage('تم الاتصال بالنواة الهندسية بنجاح.');
    } catch (error) {
      _showMessage('تعذر الاتصال بالنواة: $error', error: true);
    }
  }

  void _showMessage(String message, {bool error = false}) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(message),
          backgroundColor: error ? const Color(0xffb3261e) : null,
          behavior: SnackBarBehavior.floating,
        ),
      );
  }

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.rtl,
      child: Scaffold(
        backgroundColor: const Color(0xff07131f),
        body: SafeArea(
          child: Column(
            children: [
              _buildHeader(),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final showInspector = constraints.maxWidth >= 940;
                    return Row(
                      children: [
                        if (showInspector) _buildInspector(),
                        Expanded(child: _buildViewport()),
                      ],
                    );
                  },
                ),
              ),
              _buildStatusBar(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      height: 72,
      padding: const EdgeInsets.symmetric(horizontal: 22),
      decoration: const BoxDecoration(
        color: Color(0xff0b1d2c),
        border: Border(bottom: BorderSide(color: Color(0xff1d3547))),
      ),
      child: Row(
        children: [
          Container(
            width: 38,
            height: 38,
            decoration: BoxDecoration(
              color: const Color(0xff18a6a6).withValues(alpha: .15),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xff27c6c0)),
            ),
            child: const Icon(Icons.architecture, color: Color(0xff54e0d7)),
          ),
          const SizedBox(width: 12),
          const Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'منصة الهندسة المدنية',
                style: TextStyle(
                  color: Colors.white,
                  fontSize: 16,
                  fontWeight: FontWeight.w700,
                ),
              ),
              Text(
                'مساحة العمل الهندسية',
                style: TextStyle(color: Color(0xff8ea8b8), fontSize: 12),
              ),
            ],
          ),
          const Spacer(),
          _connectionBadge(),
          const SizedBox(width: 18),
          IconButton(
            tooltip: 'فحص اتصال النواة',
            onPressed: _checkKernel,
            icon: const Icon(Icons.sync, color: Color(0xffb7cad6)),
          ),
          const SizedBox(width: 4),
          IconButton(
            tooltip: 'الإعدادات',
            onPressed: () {},
            icon: const Icon(Icons.settings_outlined, color: Color(0xffb7cad6)),
          ),
          const SizedBox(width: 8),
          CircleAvatar(
            radius: 18,
            backgroundColor: const Color(0xff24536b),
            child: const Text(
              'م',
              style: TextStyle(
                color: Colors.white,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _connectionBadge() {
    final connected = widget.kernelReady;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
      decoration: BoxDecoration(
        color: connected ? const Color(0xff123d3b) : const Color(0xff402b25),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: connected ? const Color(0xff287d73) : const Color(0xff85533a),
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 7,
            height: 7,
            decoration: BoxDecoration(
              color: connected
                  ? const Color(0xff53d8aa)
                  : const Color(0xffffad70),
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 7),
          Text(
            connected ? 'النواة متصلة' : 'النواة غير متصلة',
            style: TextStyle(
              color: connected
                  ? const Color(0xff8de6cc)
                  : const Color(0xffffc494),
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInspector() {
    return Container(
      width: 276,
      decoration: const BoxDecoration(
        color: Color(0xff0b1d2c),
        border: Border(left: BorderSide(color: Color(0xff1d3547))),
      ),
      child: ListView(
        padding: const EdgeInsets.fromLTRB(18, 22, 18, 18),
        children: [
          const Text(
            'مستكشف المشروع',
            style: TextStyle(
              color: Colors.white,
              fontSize: 15,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 5),
          const Text(
            'مشروع تجريبي · النموذج الهندسي',
            style: TextStyle(color: Color(0xff7793a3), fontSize: 11),
          ),
          const SizedBox(height: 22),
          _treeSection(
            icon: Icons.layers_outlined,
            title: 'المستويات',
            value: '2',
            children: const ['Level 1  ·  0.00 m', 'Level 2  ·  4.00 m'],
          ),
          _treeSection(
            icon: Icons.grid_4x4,
            title: 'الشبكات',
            value: '4',
            children: const ['A', 'B', '1', '2'],
          ),
          _treeSection(
            icon: Icons.account_tree_outlined,
            title: 'العناصر الإنشائية',
            value: '7',
            children: const ['الأعمدة  ·  4', 'الكمرات  ·  4', 'البلاطة  ·  1'],
          ),
          const SizedBox(height: 18),
          Container(
            padding: const EdgeInsets.all(13),
            decoration: BoxDecoration(
              color: const Color(0xff102a3a),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xff23465b)),
            ),
            child: const Row(
              children: [
                Icon(Icons.info_outline, color: Color(0xff54e0d7), size: 18),
                SizedBox(width: 9),
                Expanded(
                  child: Text(
                    'اختر أي عنصر من المشهد لعرض خصائصه.',
                    style: TextStyle(
                      color: Color(0xffa7bdc9),
                      fontSize: 11,
                      height: 1.5,
                    ),
                  ),
                ),
              ],
            ),
          ),
          if (_selectedElement != null) ...[
            const SizedBox(height: 24),
            const Text(
              'العنصر المحدد',
              style: TextStyle(
                color: Color(0xff8ea8b8),
                fontSize: 11,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              _selectedElement!,
              style: const TextStyle(
                color: Color(0xff54e0d7),
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _treeSection({
    required IconData icon,
    required String title,
    required String value,
    required List<String> children,
  }) {
    return ExpansionTile(
      tilePadding: EdgeInsets.zero,
      childrenPadding: const EdgeInsets.only(right: 31, bottom: 8),
      initiallyExpanded: title == 'العناصر الإنشائية',
      iconColor: const Color(0xff7793a3),
      collapsedIconColor: const Color(0xff7793a3),
      title: Row(
        children: [
          Icon(icon, size: 18, color: const Color(0xff54e0d7)),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              title,
              style: const TextStyle(color: Color(0xffd5e1e7), fontSize: 13),
            ),
          ),
          Text(
            value,
            style: const TextStyle(color: Color(0xff6d8999), fontSize: 11),
          ),
        ],
      ),
      children: [
        for (final child in children)
          Align(
            alignment: Alignment.centerRight,
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Text(
                child,
                style: const TextStyle(color: Color(0xff91aab8), fontSize: 11),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildViewport() {
    return Container(
      margin: const EdgeInsets.all(14),
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: const Color(0xff091722),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xff1d3547)),
      ),
      child: Stack(
        children: [
          EngineeringViewport(
            interaction: _interaction,
            projection: _projection,
            onElementSelected: (element) {
              setState(() => _selectedElement = element);
            },
          ),
          Positioned(top: 16, right: 16, child: _viewportToolbar()),
          Positioned(top: 16, left: 16, child: _viewCube()),
          Positioned(bottom: 16, right: 16, child: _viewportLegend()),
        ],
      ),
    );
  }

  Widget _viewportToolbar() {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: const Color(0xdd0b1d2c),
        borderRadius: BorderRadius.circular(9),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _toolButton(
            icon: Icons.threesixty,
            label: 'دوران',
            active: _interaction == ViewportInteraction.orbit,
            onPressed: () =>
                setState(() => _interaction = ViewportInteraction.orbit),
          ),
          _toolButton(
            icon: Icons.pan_tool_outlined,
            label: 'تحريك',
            active: _interaction == ViewportInteraction.pan,
            onPressed: () =>
                setState(() => _interaction = ViewportInteraction.pan),
          ),
          Container(width: 1, height: 24, color: const Color(0xff2a4658)),
          _toolButton(
            icon: _projection == ViewportProjection.perspective
                ? Icons.camera_alt_outlined
                : Icons.straighten,
            label: _projection == ViewportProjection.perspective
                ? 'منظور'
                : 'متعامد',
            active: false,
            onPressed: () => setState(
              () => _projection = _projection == ViewportProjection.perspective
                  ? ViewportProjection.orthographic
                  : ViewportProjection.perspective,
            ),
          ),
          _toolButton(
            icon: Icons.center_focus_strong,
            label: 'ملاءمة',
            active: false,
            onPressed: () {},
          ),
        ],
      ),
    );
  }

  Widget _toolButton({
    required IconData icon,
    required String label,
    required bool active,
    required VoidCallback onPressed,
  }) {
    return Tooltip(
      message: label,
      child: InkWell(
        onTap: onPressed,
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 9),
          decoration: BoxDecoration(
            color: active ? const Color(0xff164b56) : Colors.transparent,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            icon,
            size: 19,
            color: active ? const Color(0xff62eee2) : const Color(0xffa5bbc6),
          ),
        ),
      ),
    );
  }

  Widget _viewCube() {
    return Container(
      width: 74,
      height: 74,
      decoration: BoxDecoration(
        color: const Color(0xc90b1d2c),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: Stack(
        alignment: Alignment.center,
        children: [
          Transform(
            alignment: Alignment.center,
            transform: Matrix4.identity()
              ..setEntry(3, 2, 0.002)
              ..rotateX(-.42)
              ..rotateY(.52),
            child: Container(
              width: 34,
              height: 34,
              decoration: BoxDecoration(
                color: const Color(0xff1b6b78),
                border: Border.all(color: const Color(0xff7fe1d8)),
              ),
              child: const Center(
                child: Text(
                  'TOP',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 8,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ),
          ),
          const Positioned(
            right: 5,
            top: 4,
            child: Text(
              'Z',
              style: TextStyle(
                color: Color(0xff65a7ff),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
          const Positioned(
            left: 5,
            bottom: 6,
            child: Text(
              'X',
              style: TextStyle(
                color: Color(0xffff8b80),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _viewportLegend() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xc90b1d2c),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _LegendDot(color: Color(0xfff06b62), label: 'X'),
          SizedBox(width: 10),
          _LegendDot(color: Color(0xff62d49b), label: 'Y'),
          SizedBox(width: 10),
          _LegendDot(color: Color(0xff65a7ff), label: 'Z'),
        ],
      ),
    );
  }

  Widget _buildStatusBar() {
    return Container(
      height: 32,
      padding: const EdgeInsets.symmetric(horizontal: 18),
      color: const Color(0xff06101a),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final compact = constraints.maxWidth < 620;
          return Row(
            children: [
              const Icon(
                Icons.mouse_outlined,
                color: Color(0xff668391),
                size: 14,
              ),
              const SizedBox(width: 7),
              Expanded(
                child: Text(
                  compact ? 'Orbit  ·  Zoom  ·  Pan' : 'اسحب للدوران  ·  عجلة الماوس للتكبير  ·  اختر التحريك لتحريك المشهد',
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: Color(0xff668391),
                    fontSize: 10,
                  ),
                ),
              ),
              if (!compact)
                const Text(
                  'الوحدات: m  ·  شبكة: 1.00',
                  style: TextStyle(color: Color(0xff668391), fontSize: 10),
                ),
            ],
          );
        },
      ),
    );
  }
}

class _LegendDot extends StatelessWidget {
  const _LegendDot({required this.color, required this.label});

  final Color color;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 7,
          height: 7,
          decoration: BoxDecoration(color: color, shape: BoxShape.circle),
        ),
        const SizedBox(width: 4),
        Text(
          label,
          style: TextStyle(
            color: color,
            fontSize: 10,
            fontWeight: FontWeight.w700,
          ),
        ),
      ],
    );
  }
}
