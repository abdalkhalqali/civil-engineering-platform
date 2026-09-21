import 'package:flutter/material.dart';

/// Dialog for creating multiple levels at once with equal spacing.
class CreateLevelsDialog extends StatefulWidget {
  const CreateLevelsDialog({super.key});

  @override
  State<CreateLevelsDialog> createState() => _CreateLevelsDialogState();
}

class _CreateLevelsDialogState extends State<CreateLevelsDialog> {
  final _countController = TextEditingController(text: '5');
  final _heightController = TextEditingController(text: '3.20');
  final _startNameController = TextEditingController(text: 'الطابق');
  String _startElevation = '0.00';

  @override
  void dispose() {
    _countController.dispose();
    _heightController.dispose();
    _startNameController.dispose();
    super.dispose();
  }

  List<_LevelPreview> _previewLevels() {
    final count = int.tryParse(_countController.text) ?? 0;
    final height = double.tryParse(_heightController.text) ?? 0;
    final startName = _startNameController.text;
    final startElev = double.tryParse(_startElevation) ?? 0;
    final suffixes = ['الأرضي', 'الأول', 'الثاني', 'الثالث', 'الرابع', 'الخامس',
      'السادس', 'السابع', 'الثامن', 'التاسع', 'العاشر'];

    final levels = <_LevelPreview>[];
    for (var i = 0; i < count; i++) {
      final elev = startElev + height * i;
      final name = i < suffixes.length
          ? '$startName ${suffixes[i]}'
          : '$startName $i';
      levels.add(_LevelPreview(name: name, elevation: elev));
    }
    return levels;
  }

  @override
  Widget build(BuildContext context) {
    final preview = _previewLevels();

    return AlertDialog(
      backgroundColor: const Color(0xff102635),
      title: const Row(
        children: [
          Icon(Icons.layers_outlined, color: Color(0xff54e0d7)),
          SizedBox(width: 10),
          Text(
            'إنشاء مستويات متعددة',
            style: TextStyle(color: Colors.white, fontSize: 16),
          ),
        ],
      ),
      content: SizedBox(
        width: 480,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'إنشاء عدد من المستويات بارتفاع ثابت وتعديلها لاحقًا.',
                style: TextStyle(color: Color(0xffa7bdc9), fontSize: 12),
              ),
              const SizedBox(height: 18),
              Row(
                children: [
                  Expanded(
                    child: _NumberField(
                      controller: _countController,
                      label: 'عدد الطوابق',
                      icon: Icons.format_list_numbered,
                      onChanged: (_) => setState(() {}),
                    ),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: _NumberField(
                      controller: _heightController,
                      label: 'ارتفاع الطابق (م)',
                      icon: Icons.height,
                      onChanged: (_) => setState(() {}),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 10),
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _startNameController,
                      decoration: _inputDecoration(
                        label: 'البداية',
                        hint: 'الطابق',
                        icon: Icons.title,
                      ),
                      onChanged: (_) => setState(() {}),
                    ),
                  ),
                  const SizedBox(width: 10),
                  Expanded(
                    child: TextField(
                      decoration: _inputDecoration(
                        label: 'المنسوب الأول (م)',
                        hint: '0.00',
                        icon: Icons.pin_drop,
                      ),
                      keyboardType: const TextInputType.numberWithOptions(decimal: true, signed: true),
                      onChanged: (v) {
                        _startElevation = v;
                        setState(() {});
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 18),
              // Preview
              Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: const Color(0xff0b1d2c),
                  borderRadius: BorderRadius.circular(8),
                  border: Border.all(color: const Color(0xff23465b)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    const Text(
                      'معاينة',
                      style: TextStyle(
                        color: Color(0xff7793a3),
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: 8),
                    for (final level in preview)
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: Row(
                          children: [
                            Container(
                              width: 3,
                              height: 14,
                              decoration: BoxDecoration(
                                color: const Color(0xff54e0d7),
                                borderRadius: BorderRadius.circular(2),
                              ),
                            ),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(
                                level.name,
                                style: const TextStyle(
                                  color: Color(0xffd5e1e7),
                                  fontSize: 12,
                                ),
                              ),
                            ),
                            Text(
                              '${level.elevation.toStringAsFixed(2)} م',
                              style: const TextStyle(
                                color: Color(0xff54e0d7),
                                fontSize: 11,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('إلغاء'),
        ),
        FilledButton.icon(
          onPressed: () {
            final count = int.tryParse(_countController.text) ?? 0;
            final height = double.tryParse(_heightController.text) ?? 0;
            final startName = _startNameController.text;
            final startElev = double.tryParse(_startElevation) ?? 0;
            if (count > 0 && height > 0) {
              Navigator.of(context).pop(_CreateLevelsResult(
                count: count,
                heightM: height,
                startName: startName,
                startElevationM: startElev,
              ));
            }
          },
          icon: const Icon(Icons.add, size: 18),
          label: const Text('إنشاء المستويات'),
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
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(8),
        borderSide: const BorderSide(color: Color(0xff54e0d7), width: 1.5),
      ),
    );
  }
}

class _LevelPreview {
  const _LevelPreview({required this.name, required this.elevation});
  final String name;
  final double elevation;
}

class _CreateLevelsResult {
  const _CreateLevelsResult({
    required this.count,
    required this.heightM,
    required this.startName,
    required this.startElevationM,
  });

  final int count;
  final double heightM;
  final String startName;
  final double startElevationM;
}

class _NumberField extends StatelessWidget {
  const _NumberField({
    required this.controller,
    required this.label,
    required this.icon,
    this.onChanged,
  });

  final TextEditingController controller;
  final String label;
  final IconData icon;
  final ValueChanged<String>? onChanged;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      keyboardType: const TextInputType.numberWithOptions(decimal: true),
      decoration: InputDecoration(
        labelText: label,
        prefixIcon: Icon(icon, color: const Color(0xff54e0d7)),
        labelStyle: const TextStyle(color: Color(0xffa7bdc9)),
        filled: true,
        fillColor: const Color(0xff0b1d2c),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xff2a4658)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xff2a4658)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xff54e0d7), width: 1.5),
        ),
      ),
      onChanged: onChanged,
    );
  }
}

/// Dialog for copying elements to other levels.
class CopyToLevelsDialog extends StatefulWidget {
  const CopyToLevelsDialog({
    required this.elementNames,
    required this.levelNames,
    super.key,
  });

  final List<String> elementNames;
  final List<String> levelNames;

  @override
  State<CopyToLevelsDialog> createState() => _CopyToLevelsDialogState();
}

class _CopyToLevelsDialogState extends State<CopyToLevelsDialog> {
  final Set<String> _selectedElements = {};
  final Set<String> _selectedLevels = {};
  bool _selectAllElements = false;
  bool _selectAllLevels = false;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      backgroundColor: const Color(0xff102635),
      title: const Row(
        children: [
          Icon(Icons.copy_all, color: Color(0xff54e0d7)),
          SizedBox(width: 10),
          Text(
            'نسخ إلى مستويات',
            style: TextStyle(color: Colors.white, fontSize: 16),
          ),
        ],
      ),
      content: SizedBox(
        width: 420,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'اختر العناصر المراد نسخها والمستويات الهدف.',
              style: TextStyle(color: Color(0xffa7bdc9), fontSize: 12),
            ),
            const SizedBox(height: 14),
            // Elements section
            _SectionHeader(
              title: 'العناصر (${_selectedElements.length}/${widget.elementNames.length})',
              selectAll: _selectAllElements,
              onSelectAll: (value) {
                setState(() {
                  _selectAllElements = value;
                  if (value) {
                    _selectedElements.addAll(widget.elementNames);
                  } else {
                    _selectedElements.clear();
                  }
                });
              },
            ),
            Container(
              height: 120,
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xff0b1d2c),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Color(0xff23465b)),
              ),
              child: ListView(
                children: [
                  for (final name in widget.elementNames)
                    CheckboxListTile(
                      value: _selectedElements.contains(name),
                      onChanged: (v) {
                        setState(() {
                          if (v == true) {
                            _selectedElements.add(name);
                          } else {
                            _selectedElements.remove(name);
                          }
                        });
                      },
                      title: Text(
                        name,
                        style: const TextStyle(
                          color: Color(0xffd5e1e7),
                          fontSize: 12,
                        ),
                      ),
                      controlAffinity: ListTileControlAffinity.leading,
                      contentPadding: EdgeInsets.zero,
                      dense: true,
                      activeColor: const Color(0xff54e0d7),
                    ),
                ],
              ),
            ),
            const SizedBox(height: 14),
            // Levels section
            _SectionHeader(
              title: 'المستويات (${_selectedLevels.length}/${widget.levelNames.length})',
              selectAll: _selectAllLevels,
              onSelectAll: (value) {
                setState(() {
                  _selectAllLevels = value;
                  if (value) {
                    _selectedLevels.addAll(widget.levelNames);
                  } else {
                    _selectedLevels.clear();
                  }
                });
              },
            ),
            Container(
              height: 100,
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xff0b1d2c),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: Color(0xff23465b)),
              ),
              child: ListView(
                children: [
                  for (final name in widget.levelNames)
                    CheckboxListTile(
                      value: _selectedLevels.contains(name),
                      onChanged: (v) {
                        setState(() {
                          if (v == true) {
                            _selectedLevels.add(name);
                          } else {
                            _selectedLevels.remove(name);
                          }
                        });
                      },
                      title: Text(
                        name,
                        style: const TextStyle(
                          color: Color(0xffd5e1e7),
                          fontSize: 12,
                        ),
                      ),
                      controlAffinity: ListTileControlAffinity.leading,
                      contentPadding: EdgeInsets.zero,
                      dense: true,
                      activeColor: const Color(0xff54e0d7),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('إلغاء'),
        ),
        FilledButton.icon(
          onPressed: _selectedElements.isNotEmpty && _selectedLevels.isNotEmpty
              ? () {
                  Navigator.of(context).pop(CopyToLevelsResult(
                    elementNames: _selectedElements.toList(),
                    levelNames: _selectedLevels.toList(),
                  ));
                }
              : null,
          icon: const Icon(Icons.copy, size: 18),
          label: const Text('نسخ'),
        ),
      ],
    );
  }
}

class CopyToLevelsResult {
  CopyToLevelsResult({
    required this.elementNames,
    required this.levelNames,
  });

  final List<String> elementNames;
  final List<String> levelNames;
}

class _SectionHeader extends StatelessWidget {
  const _SectionHeader({
    required this.title,
    required this.selectAll,
    required this.onSelectAll,
  });

  final String title;
  final bool selectAll;
  final ValueChanged<bool> onSelectAll;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Text(
          title,
          style: const TextStyle(
            color: Color(0xffd5e1e7),
            fontSize: 12,
            fontWeight: FontWeight.w600,
          ),
        ),
        const Spacer(),
        TextButton(
          onPressed: () => onSelectAll(!selectAll),
          child: Text(
            selectAll ? 'إلغاء التحديد' : 'تحديد الكل',
            style: const TextStyle(
              color: Color(0xff54e0d7),
              fontSize: 11,
            ),
          ),
        ),
      ],
    );
  }
}
