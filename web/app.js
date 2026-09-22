// Civil Engineering Platform — browser workbench.
//
// This file is a *renderer and an input device*. It owns no engineering data:
//
//   EngineeringModel (Rust/WASM) → RenderData → this file (draw + pick)
//   pointer/touch/pen → Engineering Command → Rust/WASM → model changed
//
// Nothing here computes geometry, stores element properties, or decides what a
// column is. Deleting this file and writing another one would not change the
// project by a single byte.

import * as THREE from './vendor/three.module.min.js';
import init, { WasmSession } from './pkg/geometry_kernel.js';

// --------------------------------------------------------------------- state

const app = {
  session: null,
  state: null,
  render: null,
  tool: 'select',
  selectedId: null,
  pendingStart: null, // first tap of a two-point element
  snapEnabled: true,
  drag: null,
  dirtyViewport: true,
};

const TOOLS = [
  { id: 'select', label: 'تحديد', icon: '⬚' },
  { id: 'column', label: 'عمود', icon: '▯' },
  { id: 'beam', label: 'كمرة', icon: '▭' },
  { id: 'slab', label: 'بلاطة', icon: '▤' },
  { id: 'wall', label: 'جدار', icon: '▮' },
  { id: 'foundation', label: 'أساس', icon: '⬛' },
];

const COLORS = {
  column: 0x9fb4c0,
  beam: 0xd9b26a,
  slab: 0x6fb7c9,
  wall: 0xb08e72,
  foundation: 0x8a7a68,
};

const SELECTED = 0x35d0c0;

// ------------------------------------------------------------------ three.js

const canvas = document.getElementById('viewport');
const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

const scene = new THREE.Scene();
scene.background = new THREE.Color(0x081420);
scene.fog = new THREE.Fog(0x081420, 60, 190);

const camera = new THREE.PerspectiveCamera(48, 1, 0.1, 600);

scene.add(new THREE.HemisphereLight(0xbfd8e6, 0x1b2b38, 0.9));
const key = new THREE.DirectionalLight(0xffffff, 1.15);
key.position.set(24, -18, 34);
scene.add(key);
const fill = new THREE.DirectionalLight(0x7fd6ff, 0.42);
fill.position.set(-22, 26, 14);
scene.add(fill);

const world = new THREE.Group(); // everything derived from the model
scene.add(world);

const raycaster = new THREE.Raycaster();
const pointer = new THREE.Vector2();

const orbit = { radius: 26, theta: Math.PI * 0.28, phi: Math.PI * 0.33, target: new THREE.Vector3(2, 1.5, 1.6) };

function resize() {
  const rect = canvas.parentElement.getBoundingClientRect();
  const width = Math.max(1, rect.width);
  const height = Math.max(1, rect.height);
  renderer.setSize(width, height, false);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
}

function updateCamera() {
  const { radius, theta, phi, target } = orbit;
  camera.position.set(
    target.x + radius * Math.sin(phi) * Math.sin(theta),
    target.y + radius * Math.sin(phi) * Math.cos(theta),
    target.z + radius * Math.cos(phi),
  );
  camera.up.set(0, 0, 1);
  camera.lookAt(target);
  app.dirtyViewport = true;
}

// ---------------------------------------------------------------- rendering

const meshes = [];

function clearWorld() {
  meshes.length = 0;
  while (world.children.length) {
    const child = world.children.pop();
    child.geometry?.dispose();
    if (Array.isArray(child.material)) child.material.forEach((m) => m.dispose());
    else child.material?.dispose();
  }
}

/** Builds the scene from render data — the only input this file accepts. */
function buildScene(data) {
  clearWorld();

  // Engineering grid, drawn from the model's grid lines and their extent.
  const [x0, y0, x1, y1] = data.grid_extent;
  const gridPoints = [];
  for (const grid of data.grids) {
    if (grid.direction === 'along_y') {
      gridPoints.push(grid.offset_m, y0, 0, grid.offset_m, y1, 0);
    } else {
      gridPoints.push(x0, grid.offset_m, 0, x1, grid.offset_m, 0);
    }
  }
  if (gridPoints.length) {
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.Float32BufferAttribute(gridPoints, 3));
    world.add(
      new THREE.LineSegments(
        geometry,
        new THREE.LineBasicMaterial({ color: 0x2f6d80, transparent: true, opacity: 0.85 }),
      ),
    );
  }

  // Global axes at the origin.
  const axisLength = Math.max(4, Math.abs(x1 - x0) * 0.18);
  const axes = [
    [[0, 0, 0], [axisLength, 0, 0], 0xd95b5b],
    [[0, 0, 0], [0, axisLength, 0], 0x5bd98a],
    [[0, 0, 0], [0, 0, axisLength], 0x5b8cd9],
  ];
  for (const [from, to, color] of axes) {
    const geometry = new THREE.BufferGeometry().setFromPoints([
      new THREE.Vector3(...from),
      new THREE.Vector3(...to),
    ]);
    world.add(new THREE.Line(geometry, new THREE.LineBasicMaterial({ color })));
  }

  // Linear members: columns, beams, walls are drawn as oriented solids.
  for (const member of data.members) {
    const start = new THREE.Vector3(...member.start);
    const end = new THREE.Vector3(...member.end);
    const axis = new THREE.Vector3().subVectors(end, start);
    const length = axis.length();
    if (length < 1e-6) continue;

    const geometry = new THREE.BoxGeometry(member.width_m, length, member.depth_m);
    const material = new THREE.MeshStandardMaterial({
      color: COLORS[member.category] ?? 0x9fb4c0,
      roughness: 0.78,
      metalness: 0.08,
    });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.copy(start).addScaledVector(axis, 0.5);
    mesh.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), axis.clone().normalize());
    mesh.userData.elementId = member.element_id;
    mesh.userData.name = member.name;
    mesh.userData.category = member.category;
    world.add(mesh);
    meshes.push(mesh);
  }

  // Plates: slabs, foundations and walls as plan outlines swept between elevations.
  for (const plate of data.plates) {
    if (plate.points.length < 3) continue;
    const shape = new THREE.Shape(plate.points.map(([x, y]) => new THREE.Vector2(x, y)));
    const depth = Math.max(0.01, plate.z_top - plate.z_bottom);
    const geometry = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: false });
    geometry.rotateX(-Math.PI / 2);
    geometry.translate(0, 0, plate.z_top);
    const material = new THREE.MeshStandardMaterial({
      color: COLORS[plate.category] ?? 0x6fb7c9,
      roughness: 0.8,
      metalness: 0.05,
      transparent: true,
      opacity: 0.92,
      side: THREE.DoubleSide,
    });
    const mesh = new THREE.Mesh(geometry, material);
    mesh.userData.elementId = plate.element_id;
    mesh.userData.name = plate.name;
    mesh.userData.category = plate.category;
    world.add(mesh);
    meshes.push(mesh);
  }

  // Labels (grid bubbles, level names).
  for (const label of data.labels) {
    world.add(makeLabel(label.text, label.position, label.kind === 'level' ? 0x8fd0e0 : 0x6fb0c4));
  }

  applySelection();
  updateGrip();
  app.dirtyViewport = true;
}

function makeLabel(text, position, color) {
  const scale = 3;
  const canvasEl = document.createElement('canvas');
  const ctx = canvasEl.getContext('2d');
  const font = 'bold 44px "Segoe UI", Tahoma, sans-serif';
  ctx.font = font;
  const width = Math.ceil(ctx.measureText(text).width) + 28;
  canvasEl.width = width;
  canvasEl.height = 68;
  const c = canvasEl.getContext('2d');
  c.font = font;
  c.fillStyle = 'rgba(8, 20, 32, 0.72)';
  c.fillRect(0, 0, canvasEl.width, canvasEl.height);
  c.fillStyle = `#${color.toString(16).padStart(6, '0')}`;
  c.textBaseline = 'middle';
  c.fillText(text, 14, canvasEl.height / 2);

  const texture = new THREE.CanvasTexture(canvasEl);
  texture.minFilter = THREE.LinearFilter;
  const sprite = new THREE.Sprite(
    new THREE.SpriteMaterial({ map: texture, transparent: true, depthWrite: false }),
  );
  sprite.position.set(position[0], position[1], position[2]);
  sprite.scale.set((canvasEl.width / 68) * scale * 0.5, scale * 0.5, 1);
  return sprite;
}

function applySelection() {
  for (const mesh of meshes) {
    const isSelected = mesh.userData.elementId === app.selectedId;
    if (mesh.material.emissive) {
      mesh.material.emissive.setHex(isSelected ? 0x0e6d63 : 0x000000);
      mesh.material.emissiveIntensity = isSelected ? 0.9 : 0;
    }
    const existing = mesh.children.find((child) => child.isLineSegments);
    if (isSelected && !existing) {
      const edges = new THREE.LineSegments(
        new THREE.EdgesGeometry(mesh.geometry, 25),
        new THREE.LineBasicMaterial({ color: SELECTED }),
      );
      mesh.add(edges);
    } else if (!isSelected && existing) {
      mesh.remove(existing);
      existing.geometry.dispose();
      existing.material.dispose();
    }
  }
}

// -------------------------------------------------------------------- model

function toJson(result) {
  return typeof result === 'string' ? JSON.parse(result) : result;
}

function refresh(result) {
  app.state = toJson(app.session.state());
  app.render = toJson(app.session.render_data());
  if (!app.state.levels.some((level) => level.id === app.state.active_level_id)) {
    app.state.active_level_id = app.state.levels[0]?.id ?? '';
  }
  buildScene(app.render);
  renderInspector();
  renderTools();
  renderStatus(result);
}

function selectedElement() {
  if (!app.state || !app.selectedId) return null;
  return app.state.elements.find((element) => element.id === app.selectedId) ?? null;
}

function activeLevel() {
  if (!app.state) return null;
  return app.state.levels.find((level) => level.id === app.state.active_level_id) ?? null;
}

function levelAbove(levelId) {
  const levels = app.state.levels;
  const index = levels.findIndex((level) => level.id === levelId);
  if (index < 0) return null;
  return levels[index + 1] ?? null;
}

function workElevation() {
  const level = activeLevel();
  return level ? level.elevation_m : 0;
}

function run(request) {
  const result = toJson(app.session.execute(JSON.stringify(request)));
  if (!result.ok) {
    toast(result.message || 'لم تُنفَّذ العملية', true);
    renderStatus(result);
    return result;
  }
  refresh(result);
  toast(result.label);
  return result;
}

// ------------------------------------------------------------------ picking

function ndcFromEvent(event) {
  const rect = canvas.getBoundingClientRect();
  pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
  pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
  return pointer;
}

function pickElement(event) {
  raycaster.setFromCamera(ndcFromEvent(event), camera);
  const hits = raycaster.intersectObjects(meshes, false);
  return hits.length ? hits[0].object.userData.elementId : null;
}

function workPlanePoint(event) {
  raycaster.setFromCamera(ndcFromEvent(event), camera);
  const plane = new THREE.Plane(new THREE.Vector3(0, 0, 1), -workElevation());
  const point = new THREE.Vector3();
  if (!raycaster.ray.intersectPlane(plane, point)) return null;
  return { x: point.x, y: point.y, z: workElevation() };
}

function verticalPlaneZ(event, anchor) {
  raycaster.setFromCamera(ndcFromEvent(event), camera);
  const direction = new THREE.Vector3();
  camera.getWorldDirection(direction);
  const normal = new THREE.Vector3(-direction.z, 0, direction.x);
  if (normal.lengthSq() < 1e-9) return null;
  normal.normalize();
  const plane = new THREE.Plane().setFromNormalAndCoplanarPoint(normal, anchor);
  const point = new THREE.Vector3();
  if (!raycaster.ray.intersectPlane(plane, point)) return null;
  return point.z;
}

function snapTolerance() {
  return THREE.MathUtils.clamp(orbit.radius / 45, 0.08, 1.2);
}

function snapAt(event) {
  const raw = workPlanePoint(event);
  if (!raw) return null;
  if (!app.snapEnabled) {
    return { x_m: raw.x, y_m: raw.y, z_m: raw.z, snapped: false, kind: 'free', label: '' };
  }
  return toJson(app.session.snap(raw.x, raw.y, raw.z, snapTolerance()));
}

// -------------------------------------------------------------------- tools

function chooseTool(id) {
  app.tool = id;
  app.pendingStart = null;
  hideHud();
  renderTools();
  renderStatus();
}

function onTap(event) {
  const snap = snapAt(event);
  if (!snap) return;
  const point = { x: snap.x_m, y: snap.y_m };

  switch (app.tool) {
    case 'select': {
      const id = pickElement(event);
      select(id);
      return;
    }
    case 'column': {
      const base = activeLevel();
      const top = levelAbove(base?.id);
      if (!base) return toast('لا يوجد مستوى نشط', true);
      if (!top) return toast('أنشئ مستوى أعلى أولًا', true);
      run({
        command: 'create_column',
        x_m: point.x,
        y_m: point.y,
        base_level_id: base.id,
        top_level_id: top.id,
        width_m: 0.4,
        depth_m: 0.4,
      });
      return;
    }
    case 'foundation': {
      const level = activeLevel();
      if (!level) return;
      run({
        command: 'create_foundation',
        x_m: point.x,
        y_m: point.y,
        level_id: level.id,
        thickness_m: 0.5,
        width_m: 1.5,
        depth_m: 1.5,
        foundation_kind: 'isolated',
      });
      return;
    }
    case 'beam':
    case 'wall': {
      if (!app.pendingStart) {
        app.pendingStart = point;
        toast('اختر النقطة الثانية');
        return;
      }
      const start = app.pendingStart;
      app.pendingStart = null;
      hideHud();
      const level = activeLevel();
      if (app.tool === 'beam') {
        const top = levelAbove(level?.id) ?? level;
        run({
          command: 'create_beam',
          start_x_m: start.x,
          start_y_m: start.y,
          end_x_m: point.x,
          end_y_m: point.y,
          level_id: top.id,
          width_m: 0.3,
          depth_m: 0.5,
        });
      } else {
        const top = levelAbove(level?.id);
        if (!top) return toast('أنشئ مستوى أعلى أولًا', true);
        run({
          command: 'create_wall',
          start_x_m: start.x,
          start_y_m: start.y,
          end_x_m: point.x,
          end_y_m: point.y,
          base_level_id: level.id,
          top_level_id: top.id,
          thickness_m: 0.2,
        });
      }
      return;
    }
    case 'slab': {
      if (!app.pendingStart) {
        app.pendingStart = point;
        toast('اختر الزاوية المقابلة');
        return;
      }
      const first = app.pendingStart;
      app.pendingStart = null;
      hideHud();
      const level = activeLevel();
      run({
        command: 'create_slab',
        level_id: level.id,
        thickness_m: 0.2,
        points: [first.x, first.y, point.x, first.y, point.x, point.y, first.x, point.y],
      });
      return;
    }
    default:
      return;
  }
}

function select(id) {
  app.selectedId = id;
  applySelection();
  updateGrip();
  renderInspector();
  if (window.matchMedia('(max-width: 820px)').matches && id) {
    document.getElementById('inspector').classList.add('open');
  }
}

// --------------------------------------------------------------------- grip

const grip = document.getElementById('grip');
const gripTip = document.getElementById('griptip');
let gripDrag = null;

function gripAnchor() {
  const element = selectedElement();
  if (!element) return null;
  if (!element.top_level_id) return null;
  return new THREE.Vector3(element.x_m, element.y_m, element.top_z_m);
}

function updateGrip() {
  const anchor = gripAnchor();
  if (!anchor) {
    grip.classList.remove('on');
    gripTip.classList.remove('on');
    return;
  }
  const projected = anchor.clone().project(camera);
  const rect = canvas.getBoundingClientRect();
  const x = rect.left + ((projected.x + 1) / 2) * rect.width;
  const y = rect.top + ((-projected.y + 1) / 2) * rect.height;
  grip.style.left = `${x}px`;
  grip.style.top = `${y}px`;
  grip.classList.add('on');
}

grip.addEventListener('pointerdown', (event) => {
  const element = selectedElement();
  const anchor = gripAnchor();
  if (!element || !anchor) return;
  event.preventDefault();
  grip.setPointerCapture(event.pointerId);
  grip.classList.add('dragging');
  gripDrag = { element, anchor, candidate: null, startLevelId: element.top_level_id };
  showGripTip(event, element.top_level_name, element.top_z_m);
});

grip.addEventListener('pointermove', (event) => {
  if (!gripDrag) return;
  event.preventDefault();
  const z = verticalPlaneZ(event, gripDrag.anchor);
  if (z === null) return;
  let best = null;
  for (const level of app.state.levels) {
    const distance = Math.abs(level.elevation_m - z);
    if (!best || distance < best.distance) best = { level, distance };
  }
  if (!best) return;
  gripDrag.candidate = best.level;
  showGripTip(event, best.level.name, best.level.elevation_m);
});

grip.addEventListener('pointerup', (event) => {
  if (!gripDrag) return;
  event.preventDefault();
  grip.classList.remove('dragging');
  const { candidate, element } = gripDrag;
  gripDrag = null;
  gripTip.classList.remove('on');
  if (!candidate || candidate.id === element.top_level_id) {
    toast('لم يتغير المستوى');
    return;
  }
  run({ command: 'set_top_level', element_id: element.id, top_level_id: candidate.id });
});

grip.addEventListener('pointercancel', () => {
  gripDrag = null;
  grip.classList.remove('dragging');
  gripTip.classList.remove('on');
});

function showGripTip(event, name, z) {
  gripTip.textContent = `${name} · ${z.toFixed(2)} m`;
  gripTip.style.left = `${event.clientX}px`;
  gripTip.style.top = `${event.clientY}px`;
  gripTip.classList.add('on');
}

// ---------------------------------------------------------- camera gestures

let gesture = null;

canvas.addEventListener('pointerdown', (event) => {
  canvas.setPointerCapture(event.pointerId);
  gesture = {
    pointerId: event.pointerId,
    pointerType: event.pointerType,
    startX: event.clientX,
    startY: event.clientY,
    lastX: event.clientX,
    lastY: event.clientY,
    // A pen draws; it never orbits. A finger orbits, an element or the grid is
    // still reached on tap, which is decided on release.
    moved: false,
  };
});

canvas.addEventListener('pointermove', (event) => {
  if (app.state) {
    const raw = workPlanePoint(event);
    if (raw) {
      document.getElementById('st-cursor').textContent = `${raw.x.toFixed(2)}, ${raw.y.toFixed(2)}`;
      const snap = snapAt(event);
      if (snap) {
        document.getElementById('st-snap').textContent = snap.snapped ? snap.label : 'حر';
        updateHud(event, snap, raw);
      }
    }
  }

  if (!gesture || gesture.pointerId !== event.pointerId) return;
  const dx = event.clientX - gesture.lastX;
  const dy = event.clientY - gesture.lastY;
  gesture.lastX = event.clientX;
  gesture.lastY = event.clientY;
  if (Math.abs(event.clientX - gesture.startX) > 6 || Math.abs(event.clientY - gesture.startY) > 6) {
    gesture.moved = true;
  }

  // A finger only orbits when the gesture is not a tap; a pen never does.
  const orbiting = gesture.pointerType === 'pen' ? false : gesture.moved || event.buttons === 1;
  if (!orbiting) return;

  orbit.theta -= dx * 0.006;
  orbit.phi = THREE.MathUtils.clamp(orbit.phi - dy * 0.006, 0.08, Math.PI - 0.08);
  updateCamera();
});

canvas.addEventListener('pointerup', (event) => {
  if (!gesture || gesture.pointerId !== event.pointerId) return;
  const wasTap = !gesture.moved;
  gesture = null;
  if (wasTap) onTap(event);
});

canvas.addEventListener('pointercancel', () => {
  gesture = null;
});

canvas.addEventListener(
  'wheel',
  (event) => {
    event.preventDefault();
    orbit.radius = THREE.MathUtils.clamp(orbit.radius * (1 + Math.sign(event.deltaY) * 0.1), 2, 260);
    updateCamera();
  },
  { passive: false },
);

// Two fingers: pinch to zoom, drag to pan.
let pinch = null;

canvas.addEventListener(
  'touchmove',
  (event) => {
    if (event.touches.length !== 2) {
      pinch = null;
      return;
    }
    event.preventDefault();
    const [a, b] = event.touches;
    const distance = Math.hypot(a.clientX - b.clientX, a.clientY - b.clientY);
    const center = { x: (a.clientX + b.clientX) / 2, y: (a.clientY + b.clientY) / 2 };
    if (pinch) {
      orbit.radius = THREE.MathUtils.clamp((orbit.radius * pinch.distance) / distance, 2, 260);
      const dx = center.x - pinch.center.x;
      const dy = center.y - pinch.center.y;
      const right = new THREE.Vector3().setFromMatrixColumn(camera.matrix, 0);
      const up = new THREE.Vector3().setFromMatrixColumn(camera.matrix, 1);
      const scale = orbit.radius / 500;
      orbit.target.addScaledVector(right, -dx * scale).addScaledVector(up, -dy * scale);
      updateCamera();
    }
    pinch = { distance, center };
  },
  { passive: false },
);

canvas.addEventListener('touchend', () => {
  pinch = null;
});

// ------------------------------------------------------------------ HUD

const hud = document.getElementById('hud');

function updateHud(event, snap, raw) {
  if (!app.pendingStart) {
    hud.classList.remove('on');
    return;
  }
  const dx = raw.x - app.pendingStart.x;
  const dy = raw.y - app.pendingStart.y;
  const length = Math.hypot(dx, dy);
  const angle = (Math.atan2(dy, dx) * 180) / Math.PI;
  const rows = [
    ['الطول', `${length.toFixed(2)} m`],
    ['ΔX', `${dx.toFixed(2)} m`],
    ['ΔY', `${dy.toFixed(2)} m`],
    ['الزاوية', `${angle.toFixed(1)}°`],
    ['المستوى', activeLevel()?.name ?? '—'],
  ];
  hud.innerHTML =
    rows.map(([k, v]) => `<div class="hrow"><span class="hk">${k}</span><span class="hv">${v}</span></div>`).join('') +
    `<div class="snap">${snap.snapped ? `التقاط: ${snap.label} (${snap.kind})` : 'الالتقاط: حر'}</div>`;
  hud.style.left = `${event.clientX}px`;
  hud.style.top = `${event.clientY}px`;
  hud.classList.add('on');
}

function hideHud() {
  hud.classList.remove('on');
}

// ------------------------------------------------------------------- panels

function renderTools() {
  const container = document.getElementById('tools');
  container.innerHTML = '<h2>الأدوات</h2>';
  for (const tool of TOOLS) {
    const button = document.createElement('button');
    button.className = `tool${app.tool === tool.id ? ' on' : ''}`;
    button.innerHTML = `<span class="ico">${tool.icon}</span>${tool.label}`;
    button.addEventListener('click', () => chooseTool(tool.id));
    container.appendChild(button);
  }

  const snap = document.createElement('button');
  snap.className = `tool${app.snapEnabled ? ' on' : ''}`;
  snap.innerHTML = `<span class="ico">⌖</span>الالتقاط`;
  snap.addEventListener('click', () => {
    app.snapEnabled = !app.snapEnabled;
    renderTools();
    toast(app.snapEnabled ? 'الالتقاط مفعّل' : 'الالتقاط متوقف');
  });
  container.appendChild(snap);
}

const inspector = document.getElementById('inspector');

function renderInspector() {
  if (!app.state) return;
  const element = selectedElement();
  if (!element) {
    inspector.innerHTML = `
      <h2>المشروع</h2>
      <div class="card">
        <div class="title">${app.state.name}<span class="tag">${app.state.levels.length} مستويات</span></div>
        <div class="row"><span class="k">المعرف</span><span class="v">${app.state.project_id.slice(0, 8)}</span></div>
        <div class="row"><span class="k">المراجعة</span><span class="v">${app.state.revision}</span></div>
        <div class="row"><span class="k">العناصر</span><span class="v">${app.state.elements.length}</span></div>
        <div class="row"><span class="k">الشبكات</span><span class="v">${app.state.grids.length}</span></div>
      </div>
      <div class="card">
        <div class="title">المستوى النشط</div>
        <div class="chips">${app.state.levels
          .map(
            (level) =>
              `<button class="chip${level.id === app.state.active_level_id ? ' on' : ''}" data-level="${level.id}">${level.name} · ${level.elevation_m.toFixed(2)}m</button>`,
          )
          .join('')}</div>
      </div>
      <div class="card">
        <div class="title">كيف تبني</div>
        <p class="empty">
          اختر أداة ثم المس نقطة على الشبكة. الكمرة والجدار يحتاجان نقطتين، والبلاطة زاويتين.
          بعد الإنشاء المس العنصر لتظهر مقابضه وخصائصه. المقبض العلوي ⇕ ينقل العمود إلى مستوى آخر —
          وهذا تغيير في النموذج الهندسي نفسه.
        </p>
      </div>`;
    inspector.querySelectorAll('[data-level]').forEach((chip) => {
      chip.addEventListener('click', () => {
        app.state = toJson(app.session.set_active_level(chip.dataset.level));
        if (app.state.ok === false) return toast(app.state.message, true);
        refresh();
      });
    });
    return;
  }

  const detail = toJson(app.session.element_details(element.id));
  const levels = app.state.levels;
  const levelOptions = (selected) =>
    levels
      .map((level) => `<option value="${level.id}"${level.id === selected ? ' selected' : ''}>${level.name}</option>`)
      .join('');

  const rows = [
    ['الفئة', element.category],
    ['المستوى', detail.level_name || '—'],
    ['المستوى العلوي', detail.top_level_name || '—'],
    ['المنسوب السفلي', `${detail.base_elevation_m.toFixed(3)} m`],
    ['المنسوب العلوي', `${detail.top_elevation_m.toFixed(3)} m`],
    ['الارتفاع', `${detail.height_m.toFixed(3)} m`],
    ['س', `${detail.x_m.toFixed(3)} m`],
    ['ص', `${detail.y_m.toFixed(3)} m`],
    ['القطاع', detail.section_name || `${(detail.width_m * 1000).toFixed(0)}×${(detail.depth_m * 1000).toFixed(0)} mm`],
    ['السماكة', detail.thickness_m ? `${(detail.thickness_m * 1000).toFixed(0)} mm` : '—'],
    ['الطول', detail.length_m ? `${detail.length_m.toFixed(3)} m` : '—'],
    ['المادة', detail.material_name || '—'],
    ['الدوران', `${detail.rotation_deg.toFixed(1)}°`],
  ];

  inspector.innerHTML = `
    <h2>الخصائص</h2>
    <div class="card">
      <div class="title">${element.name}<span class="tag">${element.category}</span></div>
      ${rows.map(([k, v]) => `<div class="row"><span class="k">${k}</span><span class="v">${v}</span></div>`).join('')}
    </div>

    <div class="card">
      <div class="title">تعديل هندسي</div>
      <div class="field">
        <label>قطاع مستطيل (مم) — يغيّر النموذج والهندسة معًا</label>
        <div class="grid2">
          <input id="in-w" type="number" step="10" value="${Math.round(detail.width_m * 1000) || 400}" />
          <input id="in-d" type="number" step="10" value="${Math.round(detail.depth_m * 1000) || 400}" />
        </div>
      </div>
      <div class="actions">
        <button id="btn-section">تطبيق القطاع</button>
      </div>

      ${element.top_level_id ? `
      <div class="field">
        <label>المستوى العلوي</label>
        <select id="in-top">${levelOptions(element.top_level_id)}</select>
      </div>` : ''}

      <div class="field">
        <label>المستوى السفلي</label>
        <select id="in-base">${levelOptions(element.level_id)}</select>
      </div>

      <div class="field">
        <label>إزاحة (م)</label>
        <div class="grid2">
          <input id="in-dx" type="number" step="0.1" placeholder="ΔX" value="0" />
          <input id="in-dy" type="number" step="0.1" placeholder="ΔY" value="0" />
        </div>
      </div>

      <div class="actions">
        <button id="btn-move">تحريك</button>
        <button id="btn-copy">نسخ للمستوى…</button>
      </div>
      <div class="actions">
        <button id="btn-delete" class="danger">حذف من النموذج</button>
      </div>
    </div>

    <div class="card">
      <div class="title">العلاقات</div>
      <div class="row"><span class="k">المستوى السفلي</span><span class="v">${detail.level_name || '—'}</span></div>
      <div class="row"><span class="k">المستوى العلوي</span><span class="v">${detail.top_level_name || '—'}</span></div>
      <div class="row"><span class="k">القطاع المرجعي</span><span class="v">${detail.section_id.slice(0, 8) || '—'}</span></div>
      <div class="row"><span class="k">المادة المرجعية</span><span class="v">${detail.material_id.slice(0, 8) || '—'}</span></div>
    </div>`;

  document.getElementById('btn-section').addEventListener('click', () => {
    run({
      command: 'set_section',
      element_id: element.id,
      width_mm: Number(document.getElementById('in-w').value) || 400,
      depth_mm: Number(document.getElementById('in-d').value) || 400,
    });
  });

  document.getElementById('in-top')?.addEventListener('change', (event) => {
    run({ command: 'set_top_level', element_id: element.id, top_level_id: event.target.value });
  });

  document.getElementById('in-base')?.addEventListener('change', (event) => {
    run({ command: 'set_base_level', element_id: element.id, base_level_id: event.target.value });
  });

  document.getElementById('btn-move').addEventListener('click', () => {
    run({
      command: 'move_element',
      element_id: element.id,
      dx_m: Number(document.getElementById('in-dx').value) || 0,
      dy_m: Number(document.getElementById('in-dy').value) || 0,
      dz_m: 0,
    });
  });

  document.getElementById('btn-copy').addEventListener('click', () => {
    const target = levels.find((level) => level.id !== element.top_level_id && level.id !== element.level_id);
    if (!target) return toast('لا يوجد مستوى آخر', true);
    run({ command: 'copy_to_level', element_id: element.id, top_level_id: target.id });
  });

  document.getElementById('btn-delete').addEventListener('click', () => {
    if (!confirm(`حذف ${element.name} من النموذج الهندسي؟`)) return;
    app.selectedId = null;
    run({ command: 'delete_element', element_id: element.id });
  });
}

function renderStatus(result) {
  if (!app.state) return;
  document.getElementById('st-tool').textContent = TOOLS.find((t) => t.id === app.tool)?.label ?? app.tool;
  document.getElementById('st-level').textContent = activeLevel()?.name ?? '—';
  document.getElementById('st-rev').textContent = app.state.revision;
  const canUndo = result?.can_undo ?? app.state.can_undo;
  const canRedo = result?.can_redo ?? app.state.can_redo;
  document.getElementById('btn-undo').disabled = !canUndo;
  document.getElementById('btn-redo').disabled = !canRedo;
  const state = document.getElementById('st-state');
  state.textContent = canUndo ? 'به تعديلات' : 'مشروع جديد';
  state.className = canUndo ? 'pending' : 'ok';
}

let toastTimer = null;

function toast(message, bad = false) {
  if (!message) return;
  const element = document.getElementById('toast');
  element.textContent = message;
  element.classList.toggle('bad', bad);
  element.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => element.classList.remove('show'), 2200);
}

// ------------------------------------------------------------------ actions

function newProject() {
  app.session = new WasmSession('مشروع بدون اسم', 'مبنى إنشائي', 0);
  app.selectedId = null;
  app.pendingStart = null;
  refresh();
  toast('مشروع جديد: مستويان وشبكة ونقطة أصل');
}

function saveProject() {
  const bytes = app.session.save();
  if (!bytes || !bytes.length) return toast('تعذّر الحفظ', true);
  const blob = new Blob([bytes], { type: 'application/octet-stream' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'project.civilx';
  link.click();
  URL.revokeObjectURL(url);
  toast('تم حفظ الملف — النموذج الهندسي فقط');
}

function openProject(file) {
  const reader = new FileReader();
  reader.onload = () => {
    const result = toJson(app.session.load(new Uint8Array(reader.result)));
    if (result.ok === false && result.message) return toast(result.message, true);
    app.selectedId = null;
    app.pendingStart = null;
    app.state = result;
    refresh();
    toast('أُعيد بناء النموذج من الملف');
  };
  reader.readAsArrayBuffer(file);
}

document.getElementById('btn-new').addEventListener('click', newProject);
document.getElementById('btn-save').addEventListener('click', saveProject);
document.getElementById('btn-open').addEventListener('click', () => document.getElementById('file-input').click());
document.getElementById('file-input').addEventListener('change', (event) => {
  if (event.target.files[0]) openProject(event.target.files[0]);
  event.target.value = '';
});
document.getElementById('btn-undo').addEventListener('click', () => {
  const result = toJson(app.session.undo());
  if (!result.ok) return toast(result.message, true);
  refresh(result);
  toast(result.label);
});
document.getElementById('btn-redo').addEventListener('click', () => {
  const result = toJson(app.session.redo());
  if (!result.ok) return toast(result.message, true);
  refresh(result);
  toast(result.label);
});
document.getElementById('btn-panel').addEventListener('click', () => {
  document.getElementById('inspector').classList.toggle('open');
});

window.addEventListener('keydown', (event) => {
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'z') {
    event.preventDefault();
    document.getElementById(event.shiftKey ? 'btn-redo' : 'btn-undo').click();
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault();
    saveProject();
    return;
  }
  if (event.key === 'Escape') {
    app.pendingStart = null;
    hideHud();
    select(null);
    return;
  }
  if (event.key === 'Delete' && app.selectedId) {
    document.getElementById('btn-delete')?.click();
    return;
  }
  const index = Number(event.key);
  if (index >= 1 && index <= TOOLS.length) chooseTool(TOOLS[index - 1].id);
});

window.addEventListener('resize', () => {
  resize();
  updateGrip();
});

// --------------------------------------------------------------------- boot

async function boot() {
  resize();
  updateCamera();

  await init();
  newProject();

  const data = app.render;
  if (data && data.bounds) {
    const { min, max } = data.bounds;
    orbit.target.set((min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2);
    orbit.radius = 26;
    updateCamera();
  }

  function frame() {
    requestAnimationFrame(frame);
    if (app.dirtyViewport) {
      app.dirtyViewport = false;
      updateGrip();
    }
    renderer.render(scene, camera);
  }
  frame();
}

boot().catch((error) => {
  toast(`تعذّر تحميل النواة: ${error}`, true);
  console.error(error);
});
