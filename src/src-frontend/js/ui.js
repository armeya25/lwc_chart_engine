/**
 * UI Management (Notifications, Legend, Toolbar, TrendInfo)
 */

export function showNotification(message, type = 'info', duration = 3000, textColor = null) {
    let container = document.getElementById('notification-container');
    if (!container) {
        container = document.createElement('div');
        container.id = 'notification-container';
        container.style.cssText = `position: fixed; top: 20px; right: 20px; z-index: 10000; display: flex; flex-direction: column; gap: 10px; pointer-events: none; align-items: flex-end;`;
        document.body.appendChild(container);
    }
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    let bgColor = 'rgba(40, 40, 40, 0.95)';
    let borderColor = '#555';
    let defaultTextColor = '#fff';

    if (type === 'success') { bgColor = 'rgba(46, 189, 133, 0.15)'; borderColor = '#2ebd85'; defaultTextColor = '#2ebd85'; }
    else if (type === 'error') { bgColor = 'rgba(246, 70, 93, 0.15)'; borderColor = '#f6465d'; defaultTextColor = '#f6465d'; }
    else if (type === 'warning') { bgColor = 'rgba(255, 152, 0, 0.15)'; borderColor = '#ff9800'; defaultTextColor = '#ff9800'; }

    notification.style.cssText = `background: ${bgColor}; border-left: 3px solid ${borderColor}; color: ${textColor || defaultTextColor}; padding: 12px 16px; border-radius: 4px; font-size: 13px; font-weight: 500; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); pointer-events: auto; animation: slideIn 0.3s ease-out; max-width: 350px; word-wrap: break-word;`;
    notification.textContent = message;
    container.appendChild(notification);
    setTimeout(() => {
        notification.style.animation = 'slideOut 0.3s ease-out';
        setTimeout(() => { if (notification.parentNode) notification.parentNode.removeChild(notification); }, 300);
    }, duration);
}

// Legend reference is now fetched dynamically to avoid null references if DOM isn't ready

export function toggleLegend() {
    const el = document.getElementById('legend');
    if (el) el.classList.toggle('collapsed');
}

export function addLegendItem(id, name, color) {
    const legendContent = document.getElementById('legend-content');
    if (!legendContent) return;
    
    // Legend item added, but keeping default visibility as per user request
    const legendBox = document.getElementById('legend');
    if (legendBox && legendBox.style.display === 'none') {
        // Only ensure it's not permanently 'none' if it was set via inline style, 
        // but respect the classes like 'collapsed' and 'hidden'.
    }

    let item = document.getElementById(`legend-item-${id}`);
    if (!item) {
        item = document.createElement('div');
        item.className = 'legend-item';
        item.id = `legend-item-${id}`;
        item.dataset.seriesId = id;
        item.style.cursor = 'pointer';
        legendContent.appendChild(item);
    }
    item.innerHTML = `
        <span class="legend-color" style="background-color: ${color}"></span>
        <span class="legend-name">${name}</span>
        <span class="legend-value" id="legend-val-${id}">--</span>
        <div class="legend-eye" style="margin-left: 6px; font-size: 10px; opacity: 0.7;">👁</div>
    `;

    item.addEventListener('click', () => {
        const series = window.seriesMap.get(id);
        if (series) {
            const newVisible = !series.options().visible;
            series.applyOptions({ visible: newVisible });
            item.style.opacity = newVisible ? '1' : '0.5';
        }
    });
}

export function toggleTrendInfo() {
    const el = document.getElementById('trend-info');
    if (el) el.classList.toggle('collapsed');
}

export function scrollToRealTime() {
    window.charts.forEach(c => c.timeScale().scrollToRealTime());
}

export function setupToolbar() {
    const toolbar = document.getElementById('toolbar');
    if (!toolbar) return;
    toolbar.innerHTML = `
        <div class="toolbar-trigger" id="layout-trigger"><span>Layout</span><span style="font-size: 8px; opacity: 0.7;">▼</span></div>
        <div class="dropdown-menu" id="layout-menu">
            <div class="menu-item" onclick="changeLayout('single')"><div class="layout-preview p-single"><div></div></div>Single Chart</div>
            <div class="menu-item" onclick="changeLayout('2x1')"><div class="layout-preview p-2x1"><div></div><div></div></div>2x1 Vertical</div>
            <div class="menu-item" onclick="changeLayout('1x2')"><div class="layout-preview p-1x2"><div></div><div></div></div>1x2 Horizontal</div>
            <div class="menu-item" onclick="changeLayout('1p2')"><div class="layout-preview p-1p2"><div></div><div></div><div></div></div>1 Top + 2 Bottom</div>
            <div class="menu-item" onclick="changeLayout('2x2')"><div class="layout-preview p-2x2"><div></div><div></div><div></div><div></div></div>2x2 Grid</div>
        </div>
    `;
    const trigger = document.getElementById('layout-trigger'), menu = document.getElementById('layout-menu');
    trigger.addEventListener('click', (e) => { e.stopPropagation(); menu.classList.toggle('visible'); trigger.classList.toggle('active'); });
    document.addEventListener('click', (e) => { if (!toolbar.contains(e.target)) { menu.classList.remove('visible'); trigger.classList.remove('active'); }});
}

export function changeLayout(type) {
    createLayout(type);
    const menu = document.getElementById('layout-menu'), trigger = document.getElementById('layout-trigger');
    if (menu) menu.classList.remove('visible');
    if (trigger) trigger.classList.remove('active');
}

export function createLayout(rawType) {
    const type = String(rawType || 'single').trim().toLowerCase().replace(/['"]+$/g, '').replace(/^['"]+/g, '');
    window.currentLayout = type;
    const container = document.getElementById('chart-container');
    if (container) container.className = `layout-${type}`;

    let needed = 1;
    if (type.includes('2x1') || type.includes('1x2') || type.includes('double') || type === '2') needed = 2;
    if (type.includes('1p2') || type === '3') needed = 3;
    if (type.includes('2x2') || type.includes('1p3') || type === '4') needed = 4;

    for (let i = 0; i < 4; i++) {
        const cell = document.getElementById(`chart-cell-${i}`);
        if (cell) {
            cell.style.display = i < needed ? 'block' : 'none';
        }
    }
    
    // Always trigger a resize pass
    requestAnimationFrame(() => {
        window.charts.forEach((c, id) => {
            const cell = document.getElementById(`chart-cell-${id.split('-')[1]}`);
            if (cell && cell.style.display !== 'none' && cell.clientWidth > 0) {
                c.applyOptions({ width: cell.clientWidth, height: cell.clientHeight });
            }
        });
    });
}

// Function to initialize all charts initially
export function initCharts() {
    const container = document.getElementById('chart-container');
    if (!container) return;
    
    // Make panels draggable
    makeDraggable('trade-panel', '.trade-header');
    makeDraggable('view-toolbar');
    
    for (let i = 0; i < 4; i++) {
        const chartId = `chart-${i}`;
        let cell = document.getElementById(`chart-cell-${i}`);
        if (!cell) {
            cell = document.createElement('div');
            cell.className = 'chart-cell';
            cell.id = `chart-cell-${i}`;
            container.appendChild(cell);
        }
        
        // Ensure chart exists for this cell
        if (!window.charts.has(chartId)) {
            try {
                const chart = LightweightCharts.createChart(cell, {
                    layout: { background: { color: 'transparent' }, textColor: '#d1d4dc' },
                    grid: { vertLines: { color: '#161619' }, horzLines: { color: '#161619' } },
                    crosshair: { mode: 0, horzLine: { visible: true, labelVisible: true } },
                    width: cell.clientWidth || 400,
                    height: cell.clientHeight || 300,
                    localization: { 
                        timeFormatter: (ts) => typeof ts === 'number' ? 
                            new Date(ts * 1000).toLocaleString('en-GB', { timeZone: window.chartTimeZone, day: 'numeric', month: 'short', year: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false }).replace(',', '') : ts 
                    },
                    timeScale: { 
                        timeVisible: true, secondsVisible: false, barSpacing: 10, 
                        tickMarkFormatter: (time) => { 
                            const d = typeof time === 'number' ? new Date(time * 1000) : new Date(time); 
                            return isNaN(d) ? "" : d.toLocaleDateString('en-GB', { month: 'short', day: 'numeric', timeZone: window.chartTimeZone }); 
                        } 
                    }
                });
                window.charts.set(chartId, chart);
                window.SyncManager.register(chart, cell);

                cell.onmouseenter = () => { window.activeChartId = chartId; };
                const tooltip = document.createElement('div'); tooltip.className = 'floating-tooltip'; cell.appendChild(tooltip);

                chart.subscribeCrosshairMove(param => {
                    if (!window.tooltipEnabled || !param.point || !param.time) { 
                        tooltip.style.opacity = '0'; 
                        // Reset all legend values for this chart
                        const seriesSet = window.chartSeriesMap.get(chartId);
                        if (seriesSet) seriesSet.forEach(sid => {
                            const valEl = document.getElementById(`legend-val-${sid}`);
                            if (valEl) valEl.innerText = '--';
                        });
                        return; 
                    }
                    
                    const seriesSet = window.chartSeriesMap.get(chartId);
                    if (seriesSet) {
                        seriesSet.forEach(sid => {
                            const series = window.seriesMap.get(sid);
                            const data = param.seriesData.get(series);
                            const valEl = document.getElementById(`legend-val-${sid}`);
                            if (valEl && data) {
                                let val = data.value !== undefined ? data.value : data.close;
                                if (val !== undefined) valEl.innerText = val.toFixed(2);
                            } else if (valEl) {
                                valEl.innerText = '--';
                            }
                        });
                    }

                    const ohlcId = seriesSet ? [...seriesSet][0] : null;
                    if (!ohlcId) { tooltip.style.opacity = '0'; return; }
                    const series = window.seriesMap.get(ohlcId), data = param.seriesData.get(series);
                    if (data) {
                        const dateStr = new Date(param.time * 1000).toLocaleString('en-GB', { timeZone: window.chartTimeZone, day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit', hour12: false });
                        const o = data.open || data.value, h = data.high || data.value, l = data.low || data.value, c = data.close || data.value, clr = c >= o ? 'up' : 'down';
                        tooltip.innerHTML = `<div class="tooltip-header">${series._name || 'Chart'} • ${dateStr}</div><div class="tooltip-row"><span>Open</span><span class="${clr}">${o.toFixed(2)}</span></div><div class="tooltip-row"><span>High</span><span class="${clr}">${h.toFixed(2)}</span></div><div class="tooltip-row"><span>Low</span><span class="${clr}">${l.toFixed(2)}</span></div><div class="tooltip-row"><span>Close</span><span class="${clr}">${c.toFixed(2)}</span></div>`;
                        tooltip.style.opacity = '1';
                    } else tooltip.style.opacity = '0';
                });

                if (typeof BoxManager !== 'undefined') BoxManager.init(chartId, cell);
                if (typeof PositionToolManager !== 'undefined') PositionToolManager.init(chartId, cell);
                if (typeof LineToolManager !== 'undefined') LineToolManager.init(chartId, cell);

            } catch (err) {
                console.error(`Failed to pre-initialize chart ${chartId}:`, err);
            }
        }
    }
    
    // Initial layout pass
    createLayout('single');
}

window.initCharts = initCharts;

// window.showNotification = showNotification; window.toggleLegend = toggleLegend; window.addLegendItem = addLegendItem; window.toggleTrendInfo = toggleTrendInfo; window.scrollToRealTime = scrollToRealTime; window.setupToolbar = setupToolbar; window.changeLayout = changeLayout; window.createLayout = createLayout;

// --- Trading Dashboard Logic ---
window.positionsUserHidden = true;

window.togglePanel = function(id) {
    const el = document.getElementById(id);
    if (el) {
        el.classList.toggle('hidden');
        const isNowHidden = el.classList.contains('hidden');
        
        if (id === 'positions-panel') {
            window.positionsUserHidden = isNowHidden;
            if (isNowHidden) showNotification("Positions Window Hidden", "info", 2000);
        }

        // Auto-expand trend-info when opened from toolbar
        if (id === 'trend-info' && !isNowHidden) {
            el.classList.remove('collapsed');
        }
    }
    
    // Update toolbar button states
    const btn = document.querySelector(`#view-toolbar button[onclick*="${id}"]`);
    if (btn) {
        const el = document.getElementById(id);
        const isActive = el && !el.classList.contains('hidden') && !el.classList.contains('collapsed');
        btn.style.borderColor = isActive ? 'var(--primary-color)' : '';
        btn.style.color = isActive ? 'var(--primary-color)' : '';
    }
};

window.executeTrade = function(side) {
    const qtyInput = document.getElementById('trade-qty');
    const tpInput = document.getElementById('trade-tp');
    const slInput = document.getElementById('trade-sl');
    
    const qty = parseFloat(qtyInput ? qtyInput.value : 1);
    const tp = tpInput ? parseFloat(tpInput.value) : null;
    const sl = slInput ? parseFloat(slInput.value) : null;
    
    if (window.__TAURI__) {
        const invoke = window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke;
        if (invoke) {
            invoke('emit_to_backend', { 
                action: 'trade', 
                data: { 
                    side: side, 
                    qty: qty,
                    tp: tp,
                    sl: sl
                } 
            })
                .then(() => showNotification(`Position: ${side.toUpperCase()} ${qty} | TP: ${tp||'-'} SL: ${sl||'-'}`, 'success'))
                .catch(e => {
                    console.error("Trade execution failed:", e);
                    showNotification(`Execution Failed: ${e}`, 'error');
                });
        }
    } else {
        console.log("Mock Trade:", side, qty, tp, sl);
        showNotification(`Mock: ${side.toUpperCase()} ${qty}`, 'warning');
    }
};

export function updatePositionsUI(positions) {
    const panel = document.getElementById('positions-panel');
    const body = document.getElementById('positions-body');
    if (!panel || !body) return;

    // If user explicitly manually hid it, keep it hidden.
    if (window.positionsUserHidden) {
        panel.classList.add('hidden');
        return;
    }

    if (!positions || positions.length === 0) {
        panel.classList.add('hidden');
        return;
    }

    panel.classList.remove('hidden');
    body.innerHTML = '';
    
    positions.forEach(p => {
        const tr = document.createElement('tr');
        const pnl = p.pnl || 0;
        const pnlClass = pnl >= 0 ? 'pos-pnl-up' : 'pos-pnl-down';
        const sideClass = p.side === 'buy' ? 'pos-side-buy' : 'pos-side-sell';
        
        tr.innerHTML = `
            <td class="${sideClass}">${p.side.upperCase ? p.side.upperCase() : p.side.toUpperCase()}</td>
            <td>${p.qty}</td>
            <td>${p.entry.toFixed(2)}</td>
            <td>${p.price.toFixed(2)}</td>
            <td>${p.tp ? p.tp.toFixed(2) : '-'}</td>
            <td>${p.sl ? p.sl.toFixed(2) : '-'}</td>
            <td class="${pnlClass}">${pnl >= 0 ? '+' : ''}${pnl.toFixed(2)}</td>
        `;
        body.appendChild(tr);
    });
};

function makeDraggable(elementId, handleSelector) {
    const element = document.getElementById(elementId);
    if (!element) return;
    const handle = element.querySelector(handleSelector) || element;
    
    let pos1 = 0, pos2 = 0, pos3 = 0, pos4 = 0;
    handle.onmousedown = dragMouseDown;

    function dragMouseDown(e) {
        e.preventDefault();
        pos3 = e.clientX;
        pos4 = e.clientY;
        document.onmouseup = closeDragElement;
        document.onmousemove = elementDrag;
    }

    function elementDrag(e) {
        e.preventDefault();
        pos1 = pos3 - e.clientX;
        pos2 = pos4 - e.clientY;
        pos3 = e.clientX;
        pos4 = e.clientY;
        element.style.top = (element.offsetTop - pos2) + "px";
        element.style.left = (element.offsetLeft - pos1) + "px";
        element.style.right = 'auto';
        element.style.bottom = 'auto';
    }

    function closeDragElement() {
        document.onmouseup = null;
        document.onmousemove = null;
    }
}

// Global Toolbar Actions
function toggleScaleControls() {
    const toolbar = document.getElementById('scale-toolbar');
    if (toolbar) toolbar.classList.toggle('hidden');
}

function setScaleMode(mode) {
    const chart = window.charts.get(window.activeChartId);
    if (!chart) return;
    
    const priceScale = chart.priceScale('right');
    const buttons = document.querySelectorAll('.scale-btn');
    buttons.forEach(b => b.classList.remove('active'));
    
    // Find and activate the correct button
    const btn = Array.from(buttons).find(b => b.getAttribute('onclick').includes(mode));
    if (btn) btn.classList.add('active');

    if (mode === 'auto') {
        priceScale.applyOptions({ mode: 0, autoScale: true });
    } else if (mode === 'log') {
        priceScale.applyOptions({ mode: 1 });
    } else if (mode === 'pct') {
        priceScale.applyOptions({ mode: 2 });
    }
}
