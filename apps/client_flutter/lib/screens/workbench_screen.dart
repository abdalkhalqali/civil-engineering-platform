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
  String _projectName = 'مشروع تجريبي';
  String _projectType = 'مبنى إنشائي';
  double _landArea = 1200;

  static const _projectTypes = [
    'مبنى إنشائي',
    'جسر',
    'أعمال ترابية وتهيئة أرض',
  ];

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

  Future<void> _openNewProjectDialog() async {
    final draft = await showDialog<_ProjectDraft>(
      context: context,
      builder: (context) => _NewProjectDialog(
        initialName: _projectName,
        initialType: _projectType,
        initialLandArea: _landArea,
        projectTypes: _projectTypes,
      ),
    );
    if (draft == null || !mounted) return;

    setState(() {
      _projectName = draft.name;
      _projectType = draft.type;
      _landArea = draft.landArea;
      _selectedElement = null;
    });

    if (!widget.kernelReady) {
      _showMessage(
        'تم تجهيز المشروع في الواجهة. اتصل بالنواة لحفظه في النموذج الهندسي.',
        error: true,
      );
      return;
    }

    try {
      final summary = createProject(name: draft.name);
      _showMessage(
        'تم إنشاء "${summary.name}" بنجاح · مساحة الأرض ${_formatArea(draft.landArea)} م²',
      );
    } catch (error) {
      _showMessage('تعذر إنشاء المشروع في النواة: $error', error: true);
    }
  }

  String _formatArea(double area) {
    return area == area.roundToDouble()
        ? area.toStringAsFixed(0)
        : area.toStringAsFixed(2);
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
    return LayoutBuilder(
      builder: (context, constraints) {
        final compact = constraints.maxWidth < 720;
        return Container(
          height: 72,
          padding: EdgeInsets.symmetric(horizontal: compact ? 10 : 22),
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
              if (!compact) ...[
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
              ],
              const Spacer(),
              if (!compact) _connectionBadge(),
              IconButton(
                tooltip: 'إنشاء مشروع جديد',
                onPressed: _openNewProjectDialog,
                icon: const Icon(
                  Icons.create_new_folder_outlined,
                  color: Color(0xffb7cad6),
                ),
              ),
              if (!compact) ...[
                const SizedBox(width: 4),
                IconButton(
                  tooltip: 'فحص اتصال النواة',
                  onPressed: _checkKernel,
                  icon: const Icon(Icons.sync, color: Color(0xffb7cad6)),
                ),
                const SizedBox(width: 4),
                IconButton(
                  tooltip: 'الإعدادات',
                  onPressed: () {},
                  icon: const Icon(
                    Icons.settings_outlined,
                    color: Color(0xffb7cad6),
                  ),
                ),
              ],
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
      },
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
          Text(
            '$_projectType · مساحة ${_formatArea(_landArea)} م²',
            style: TextStyle(color: Color(0xff7793a3), fontSize: 11),
          ),
          const SizedBox(height: 4),
          Text(
            _projectName,
            style: const TextStyle(
              color: Color(0xffd5e1e7),
              fontSize: 13,
              fontWeight: FontWeight.w600,
            ),
            overflow: TextOverflow.ellipsis,
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

class _ProjectDraft {
  const _ProjectDraft({
    required this.name,
    required this.type,
    required this.landArea,
  });

  final String name;
  final String type;
  final double landArea;
}

class _NewProjectDialog extends StatefulWidget {
  const _NewProjectDialog({
    required this.initialName,
    required this.initialType,
    required this.initialLandArea,
    required this.projectTypes,
  });

  final String initialName;
  final String initialType;
  final double initialLandArea;
  final List<String> projectTypes;

  @override
  State<_NewProjectDialog> createState() => _NewProjectDialogState();
}

class _NewProjectDialogState extends State<_NewProjectDialog> {
  late final TextEditingController _nameController;
  late final TextEditingController _areaController;
  late String _selectedType;
  String? _errorText;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.initialName);
    _areaController = TextEditingController(
      text: widget.initialLandArea.toStringAsFixed(0),
    );
    _selectedType = widget.initialType;
  }

  @override
  void dispose() {
    _nameController.dispose();
    _areaController.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _nameController.text.trim();
    final area = double.tryParse(_areaController.text.trim());
    if (name.isEmpty) {
      setState(() => _errorText = 'اكتب اسمًا للمشروع أولًا.');
      return;
    }
    if (area == null || area <= 0) {
      setState(() => _errorText = 'أدخل مساحة صحيحة أكبر من صفر.');
      return;
    }
    Navigator.of(context)
        .pop(_ProjectDraft(name: name, type: _selectedType, landArea: area));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final compact = MediaQuery.sizeOf(context).width < 520;
    return AlertDialog(
      backgroundColor: const Color(0xff102635),
      surfaceTintColor: Colors.transparent,
      title: const Row(
        children: [
          Icon(Icons.create_new_folder_outlined, color: Color(0xff54e0d7)),
          SizedBox(width: 10),
          Text(
            'إنشاء مشروع جديد',
            style: TextStyle(color: Colors.white, fontSize: 18),
          ),
        ],
      ),
      content: SizedBox(
        width: compact ? null : 480,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'أدخل بيانات المشروع الأساسية للبدء في مساحة عمل جديدة.',
                style: TextStyle(color: Color(0xffa7bdc9), fontSize: 12),
              ),
              const SizedBox(height: 22),
              TextField(
                controller: _nameController,
                autofocus: true,
                textInputAction: TextInputAction.next,
                decoration: _inputDecoration(
                  label: 'اسم المشروع',
                  hint: 'مثال: جسر وادي حضرموت',
                  icon: Icons.title,
                ),
              ),
              const SizedBox(height: 14),
              DropdownButtonFormField<String>(
                initialValue: _selectedType,
                dropdownColor: const Color(0xff183344),
                decoration: _inputDecoration(
                  label: 'نوع المشروع',
                  hint: '',
                  icon: Icons.category_outlined,
                ),
                items: [
                  for (final type in widget.projectTypes)
                    DropdownMenuItem(
                      value: type,
                      child: Text(
                        type,
                        style: const TextStyle(color: Colors.white),
                      ),
                    ),
                ],
                onChanged: (value) {
                  if (value != null) setState(() => _selectedType = value);
                },
              ),
              const SizedBox(height: 14),
              TextField(
                controller: _areaController,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => _submit(),
                decoration: _inputDecoration(
                  label: 'المساحة التقديرية لقطعة الأرض',
                  hint: '1200',
                  icon: Icons.square_foot,
                ).copyWith(suffixText: 'م²'),
              ),
              if (_errorText != null) ...[
                const SizedBox(height: 12),
                Text(
                  _errorText!,
                  style: TextStyle(
                    color: theme.colorScheme.error,
                    fontSize: 12,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
      actionsPadding: const EdgeInsets.fromLTRB(20, 0, 20, 18),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('إلغاء'),
        ),
        FilledButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.add, size: 18),
          label: const Text('إنشاء المشروع'),
        ),
      ],
    );
  }

  InputDecoration _inputDecoration({
    required String label,
    required String hint,
    required IconData icon,
  }) {
    return InputDecoration(
      labelText: label,
      hintText: hint,
      prefixIcon: Icon(icon, color: const Color(0xff54e0d7)),
      labelStyle: const TextStyle(color: Color(0xffa7bdc9)),
      hintStyle: const TextStyle(color: Color(0xff6f8c9d)),
      filled: true,
      fillColor: const Color(0xff0b1d2c),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff54e0d7), width: 1.5),
      ),
    );
  }
}
