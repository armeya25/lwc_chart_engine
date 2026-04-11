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

export function hideLoader() {
    const el = document.getElementById('loading');
    if (el) {
        el.style.opacity = '0';
        setTimeout(() => { if (el.parentNode) el.parentNode.removeChild(el); }, 500);
    }
}

// Legend reference is now fetched dynamically to avoid null references if DOM isn't ready

export function toggleLegend() {
    document.querySelectorAll('.chart-legend').forEach(el => {
        el.classList.toggle('collapsed');
    });
}

export function addLegendItem(chartId, id, name, color, type = 'line', indicatorName = null, humanName = null, indicatorTypeName = null) {
    const cell = document.getElementById(chartId.replace('chart-', 'chart-cell-'));
    if (!cell) return;
    const legendContent = cell.querySelector('.legend-content');
    if (!legendContent) return;
    
    const isCandle = type === 'candle' || type === 'candlestick';
    const isVolume = name && name.toLowerCase().includes('volume');

    // Helper to get indicator parameters label
    const getParamsLabel = (indName) => {
        if (!indName || !window.indicatorsMetadata) return '';
        const meta = window.indicatorsMetadata.get(indName);
        if (!meta || !meta.params) return '';
        const vals = Object.values(meta.params).filter(v => typeof v === 'number' || typeof v === 'string');
        return vals.length > 0 ? ` (${vals.join(', ')})` : '';
    };

    // 1. Handle Grouped Indicators (Smart Grouping)
    if (indicatorName) {
        const groupDomId = `${chartId}-legend-group-${indicatorName}`;
        let group = document.getElementById(groupDomId);
        const paramsLabel = getParamsLabel(indicatorName);
        const groupLabel = (indicatorTypeName || indicatorName) + paramsLabel;
        const seriesLabel = humanName || name;
        
        if (group) {
            const row = group.querySelector('.legend-group-header');
            if (row) {
                const nameEl = row.querySelector('.legend-name');
                if (nameEl && (indicatorTypeName || paramsLabel)) nameEl.textContent = groupLabel;
            }
            const item = document.getElementById(`${chartId}-legend-item-${id}`);
            if (item) {
                const labelSpan = item.querySelector('.legend-sub-label');
                if (labelSpan && humanName) labelSpan.textContent = `${humanName}:`;
            }
        }

        if (!group) {
            group = document.createElement('div');
            group.id = groupDomId;
            group.className = 'legend-group';
            group.dataset.indicator = indicatorName;
            group.dataset.seriesCount = 1;
            group.dataset.mainSid = id; 
            
            group.innerHTML = `
                <div class="legend-group-header" id="${chartId}-group-row-${indicatorName}">
                    <div class="legend-header-main">
                        <span class="legend-name" title="${groupLabel}">${groupLabel}</span>
                        <span class="legend-value main-val" id="${chartId}-legend-val-${id}">--</span>
                    </div>
                    <div class="legend-header-actions">
                        <span class="legend-visibility-btn" title="Toggle Visibility" onclick="event.stopPropagation(); window.toggleIndicatorVisibility('${indicatorName}', '${chartId}')">👁️</span>
                        <span class="legend-settings-btn" title="Settings" onclick="event.stopPropagation(); window.showIndicatorSettings('${indicatorName}')">⚙️</span>
                        <span class="legend-close" title="Remove" onclick="event.stopPropagation(); window.CommandHandlers.remove_indicator('${indicatorName}', '${chartId}')">×</span>
                    </div>
                </div>
                <div class="legend-group-content" id="${chartId}-group-content-${indicatorName}"></div>
            `;
            legendContent.appendChild(group);
            return;
        }

        const content = group.querySelector('.legend-group-content');
        let count = parseInt(group.dataset.seriesCount);
        
        if (count === 1) {
            if (content) content.classList.remove('hidden');
        }
        
        if (id !== group.dataset.mainSid && !document.getElementById(`${chartId}-legend-item-${id}`)) {
            group.dataset.seriesCount = count + 1;
            const item = document.createElement('div');
            item.id = `${chartId}-legend-item-${id}`;
            item.className = 'legend-sub-item';
            item.dataset.seriesId = id;
            item.innerHTML = `
                <div style="display: flex; align-items: center; font-size: 11px; padding: 1px 0;">
                    <span class="legend-color" style="background-color: ${color}; width: 6px; height: 6px; margin-right: 6px; border-radius: 50%;"></span>
                    <span class="legend-sub-label" style="color: var(--text-secondary); margin-right: 4px;">${seriesLabel}:</span>
                    <span class="legend-value" id="${chartId}-legend-val-${id}">--</span>
                </div>
            `;
            content.appendChild(item);
        }
        return;
    }

    // 2. Handle Regular Series or Main Status Line
    const itemDomId = `${chartId}-legend-item-${id}`;
    let item = document.getElementById(itemDomId);
    if (!item) {
        item = document.createElement('div');
        item.id = itemDomId;
        item.dataset.seriesId = id;
        legendContent.prepend(item); // Put main series at top
    }

    item.className = isCandle ? 'legend-item candle-item' : 'legend-item';
    
    if (isCandle) {
        item.innerHTML = `
            <div class="legend-header-main">
                <span class="legend-name">${name}</span>
                <div class="legend-status-line" id="${chartId}-status-${id}">
                    <div class="status-item"><span class="status-label">O</span><span class="status-val o">--</span></div>
                    <div class="status-item"><span class="status-label">H</span><span class="status-val h">--</span></div>
                    <div class="status-item"><span class="status-label">L</span><span class="status-val l">--</span></div>
                    <div class="status-item"><span class="status-label">C</span><span class="status-val c">--</span></div>
                    <div class="status-item"><span class="status-val chg">--</span> <span class="status-val pct">(--)</span></div>
                </div>
            </div>
            <div class="legend-header-actions">
                <span class="legend-visibility-btn" title="Toggle Visibility" onclick="event.stopPropagation(); window.toggleSeriesVisibility('${id}', '${chartId}')">👁️</span>
            </div>
        `;
    } else {
        item.innerHTML = `
            <div class="legend-header-main">
                <span class="legend-color" style="background-color: ${color}; box-shadow: 0 0 5px ${color}"></span>
                <span class="legend-name">${name}</span>
                <span class="legend-value" id="${chartId}-legend-val-${id}">--</span>
            </div>
            <div class="legend-header-actions">
                <span class="legend-visibility-btn" title="Toggle Visibility" onclick="event.stopPropagation(); window.toggleSeriesVisibility('${id}', '${chartId}')">👁️</span>
                <span class="legend-settings-btn" title="Settings" onclick="event.stopPropagation(); window.showIndicatorSettings('${name}')">⚙️</span>
                <span class="legend-close" title="Remove" onclick="event.stopPropagation(); window.CommandHandlers.remove_series(window.charts.get('${chartId}'), {seriesId: '${id}'}, '${chartId}')">×</span>
            </div>
        `;
    }
}

window.toggleSeriesVisibility = function(seriesId, chartId) {
    const series = window.seriesMap.get(seriesId);
    if (!series) return;
    
    const isVisible = series.options().visible;
    const newVisible = !isVisible;
    series.applyOptions({ visible: newVisible });
    
    const itemDomId = `${chartId || ''}-legend-item-${seriesId}`;
    const item = document.getElementById(itemDomId);
    if (item) {
        const eye = item.querySelector('.legend-visibility-btn');
        if (eye) eye.style.opacity = newVisible ? '1' : '0.3';
        item.style.opacity = newVisible ? '1' : '0.6';
    }
};

window.toggleIndicatorVisibility = function(indicatorName, chartId) {
    const groupDomId = `${chartId || ''}-legend-group-${indicatorName}`;
    const group = document.getElementById(groupDomId);
    if (!group) return;
    
    const content = group.querySelector('.legend-group-content');
    const sids = Array.from(content.querySelectorAll('.legend-sub-item')).map(el => el.dataset.seriesId);
    
    // Also include children from single row if it hasn't been promoted
    if (group.dataset.seriesCount == 1) {
        const sid = group.dataset.mainSid;
        if (sid && !sids.includes(sid)) sids.push(sid);
    }

    let firstVisible = true;
    if (sids.length > 0) {
        const firstSeries = window.seriesMap.get(sids[0]);
        if (firstSeries) firstVisible = firstSeries.options().visible;
    }
    const newVisible = !firstVisible;
    sids.forEach(sid => {
        const series = window.seriesMap.get(sid);
        if (series) series.applyOptions({ visible: newVisible });
    });
    const eye = group.querySelector('.legend-visibility-btn');
    if (eye) eye.style.opacity = newVisible ? '1' : '0.3';
    group.style.opacity = newVisible ? '1' : '0.6';
    
    // Trigger pane layout rebalance for all charts
    if (window.charts) {
        window.charts.forEach(chart => {
            if (window.CommandHandlers && window.CommandHandlers.ensurePaneLayout) {
                window.CommandHandlers.ensurePaneLayout(chart);
            }
        });
    }
};


export function toggleTrendInfo() {
    const el = document.getElementById('trend-info');
    if (el) el.classList.toggle('collapsed');
}

export function scrollToRealTime() {
    window.charts.forEach(c => c.timeScale().scrollToRealTime());
}

export function changeLayout(type) {
    createLayout(type);
    
    // Update active state in View -> Layout submenu
    const submenu = document.querySelector('.submenu');
    if (submenu) {
        submenu.querySelectorAll('.menu-item').forEach(item => {
            const itemOnClick = item.getAttribute('onclick');
            if (itemOnClick && itemOnClick.includes(`'${type}'`)) {
                item.classList.add('active');
            } else {
                item.classList.remove('active');
            }
        });
    }

    // Auto-close the main dropdown
    const menu = document.getElementById('view-menu-dropdown');
    const trigger = document.getElementById('view-menu-trigger');
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
    
    // Resize pass is now handled by ResizeObserver, but we can trigger 
    // a manual resize update to ensure immediate alignment on layout switch.
    window.charts.forEach((c, id) => {
        const cell = document.getElementById(`chart-cell-${id.split('-')[1]}`);
        if (cell && cell.style.display !== 'none' && cell.clientWidth > 0) {
            c.applyOptions({ width: cell.clientWidth, height: cell.clientHeight });
        }
    });
}

// Function to initialize all charts initially
export function initCharts() {
    const container = document.getElementById('chart-container');
    if (!container) return;
    
    // Make panels draggable
    makeDraggable('trade-panel', '.trade-header');
    
    for (let i = 0; i < 4; i++) {
        const chartId = `chart-${i}`;
        let cell = document.getElementById(`chart-cell-${i}`);
        if (!cell) {
            cell = document.createElement('div');
            cell.className = 'chart-cell';
            cell.id = `chart-cell-${i}`;
            
            // Inject per-chart legend overlay
            const legendObj = document.createElement('div');
            legendObj.className = 'chart-legend';
            legendObj.id = `${chartId}-legend`;
            legendObj.innerHTML = `<div class="legend-content"></div>`;
            cell.appendChild(legendObj);
            
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

                cell.onmouseenter = () => { 
                    window.activeChartId = chartId;
                    document.querySelectorAll('.chart-cell').forEach(c => c.classList.remove('active'));
                    cell.classList.add('active');
                };
                const tooltip = document.createElement('div'); tooltip.className = 'floating-tooltip'; cell.appendChild(tooltip);

                // Use ResizeObserver for robust layout handling
                const resizeObserver = new ResizeObserver(entries => {
                    for (let entry of entries) {
                        const { width, height } = entry.contentRect;
                        if (width > 0 && height > 0) {
                            chart.applyOptions({ width, height });
                        }
                    }
                });
                resizeObserver.observe(cell);

                chart.subscribeCrosshairMove(param => {
                    const seriesSet = window.chartSeriesMap.get(chartId);
                    
                    // 1. Legend Updates (Always update when crosshair is on chart)
                    if (param.time && seriesSet) {
                        const logicalIndex = param.logical !== undefined
                            ? Math.round(param.logical)
                            : chart.timeScale().coordinateToLogical(param.point ? param.point.x : 0);

                        seriesSet.forEach(sid => {
                            const series = window.seriesMap.get(sid);
                            if (!series) return;

                            const statusEl = cell.querySelector(`#${chartId}-status-${sid}`);
                            const valEl = cell.querySelector(`#${chartId}-legend-val-${sid}`);
                            
                            // Get Data
                            let data = param.seriesData.get(series);
                            if (!data && logicalIndex != null && logicalIndex >= 0) {
                                try { data = series.dataByIndex(logicalIndex, -1); } catch (_) {}
                            }

                            if (statusEl && data && (data.open !== undefined || data.value !== undefined)) {
                                // Detailed OHLC update for candles
                                const o = data.open ?? data.value, h = data.high ?? data.value, l = data.low ?? data.value, c = data.close ?? data.value;
                                const diff = c - o, pct = (diff / o) * 100, clrClass = diff >= 0 ? 'up' : 'down';
                                
                                const updateVal = (cls, val) => {
                                    const el = statusEl.querySelector(`.status-val.${cls}`);
                                    if (el) {
                                        el.innerText = val.toFixed(2);
                                        el.className = `status-val ${cls} ${clrClass}`;
                                    }
                                };
                                updateVal('o', o); updateVal('h', h); updateVal('l', l); updateVal('c', c);
                                
                                const chgEl = statusEl.querySelector('.status-val.chg');
                                const pctEl = statusEl.querySelector('.status-val.pct');
                                if (chgEl) { chgEl.innerText = (diff >= 0 ? '+' : '') + diff.toFixed(2); chgEl.className = `status-val chg ${clrClass}`; }
                                if (pctEl) { pctEl.innerText = `(${(diff >= 0 ? '+' : '') + pct.toFixed(2)}%)`; pctEl.className = `status-val pct ${clrClass}`; }
                                
                            } else if (valEl) {
                                // Simple value update for lines/indicators
                                if (data) {
                                    const val = data.value !== undefined ? data.value : data.close;
                                    valEl.innerText = val !== undefined ? val.toFixed(2) : '--';
                                } else {
                                    valEl.innerText = '--';
                                }
                            }
                        });
                    } else if (seriesSet) {
                        // Reset to -- when crosshair leaves
                        seriesSet.forEach(sid => {
                            const statusEl = cell.querySelector(`#${chartId}-status-${sid}`);
                            const valEl = cell.querySelector(`#${chartId}-legend-val-${sid}`);
                            if (statusEl) {
                                statusEl.querySelectorAll('.status-val').forEach(el => {
                                    el.innerText = el.classList.contains('pct') ? '(--)' : '--';
                                    el.classList.remove('up', 'down');
                                });
                            }
                            if (valEl) valEl.innerText = '--';
                        });
                    }

                    // 2. Floating Tooltip (Only if enabled and crosshair is on chart)
                    if (!window.tooltipEnabled || !param.point || !param.time) { 
                        tooltip.style.opacity = '0'; 
                        return; 
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

    // Setup Window Controls
    if (window.__TAURI__) {
        setupWindowControls();
        setupViewMenu();
    }
}

function setupViewMenu() {
    const trigger = document.getElementById('view-menu-trigger');
    const menu = document.getElementById('view-menu-dropdown');
    
    if (!trigger || !menu) return;
    
    trigger.onclick = (e) => {
        e.stopPropagation();
        menu.classList.toggle('visible');
    };
    
    document.addEventListener('click', (e) => {
        if (!menu.contains(e.target) && e.target !== trigger) {
            menu.classList.remove('visible');
        }
    });

    // Keyboard Shortcuts
    document.addEventListener('keydown', (e) => {
        const key = e.key.toLowerCase();
        if (e.ctrlKey || e.metaKey) return; // Don't trigger on ctrl+C etc.
        
        if (key === 'l') togglePanel('legend');
        if (key === 'e') togglePanel('trade-panel');
        if (key === 'p') togglePanel('positions-panel');
        if (key === 'i') togglePanel('trend-info');
    });
}


function setupWindowControls() {
    console.log("Setting up window controls...");
    if (!window.__TAURI__) {
        console.warn("Tauri context not found. Window controls disabled.");
        return;
    }

    const { getCurrentWindow } = window.__TAURI__.window;
    if (!getCurrentWindow) {
        console.error("getCurrentWindow not found in window.__TAURI__.window");
        return;
    }

    const appWindow = getCurrentWindow();
    console.log("App window handle retrieved:", appWindow.label || "main");

    const minBtn = document.getElementById('titlebar-minimize');
    const maxBtn = document.getElementById('titlebar-maximize');
    const closeBtn = document.getElementById('titlebar-close');

    if (minBtn) {
        minBtn.onclick = async () => {
            console.log("Minimize clicked");
            await appWindow.minimize();
        };
    }
    
    if (maxBtn) {
        maxBtn.onclick = async () => {
            console.log("Toggle Maximize clicked");
            await appWindow.toggleMaximize();
        };
    }
    
    if (closeBtn) {
        closeBtn.onclick = async () => {
            console.log("Window close requested");
            try {
                await appWindow.close();
            } catch (err) {
                console.error("Failed to close window:", err);
            }
        };
    }

    // Maximize icon toggle script 
    appWindow.onResized(async () => {
        const isMaximized = await appWindow.isMaximized();
        if (maxBtn) {
            maxBtn.innerHTML = isMaximized 
                ? '<svg width="12" height="12" viewBox="0 0 24 24"><path fill="currentColor" d="M4 8h4V4h12v12h-4v4H4V8zm12 0v6h2V6H10v2h6zM6 10v8h8v-8H6z"/></svg>' // Restore icon
                : '<svg width="12" height="12" viewBox="0 0 24 24"><path fill="currentColor" d="M4 4h16v16H4V4zm2 4v10h12V8H6z"/></svg>'; // Maximize icon
        }
    });
}


// --- Trading Dashboard Logic ---
window.positionsUserHidden = true;

window.togglePanel = function(id) {
    if (id === 'legend') {
        toggleLegend();
        return;
    }
    const el = document.getElementById(id);
    if (el) {
        el.classList.toggle('hidden');
        const isNowHidden = el.classList.contains('hidden');
        
        if (id === 'positions-panel') {
            window.positionsUserHidden = isNowHidden;
            if (isNowHidden) showNotification("Positions Window Hidden", "info", 2000);
        }

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
    }
}

export function updatePositionsUI(positions) {
    const panel = document.getElementById('positions-panel');
    const body = document.getElementById('positions-body');
    if (!panel || !body) return;

    // If user explicitly manually hid it, keep it hidden.
    if (window.positionsUserHidden) {
        panel.classList.add('hidden');
        return;
    }

    // Show panel if we have positions
    if (positions && positions.length > 0) {
        panel.classList.remove('hidden');
    }

    body.innerHTML = '';
    let totalPnl = 0;
    
    if (positions) {
        positions.forEach(p => {
            totalPnl += p.pnl;
            const tr = document.createElement('tr');
            const pnlClass = p.pnl >= 0 ? 'pnl-up' : 'pnl-down';
            const sideClass = p.side === 'buy' ? 'side-buy' : 'side-sell';
            
            tr.innerHTML = `
                <td><span class="${sideClass}">${p.side.toUpperCase()}</span></td>
                <td>${p.qty}</td>
                <td>${p.entry.toFixed(2)}</td>
                <td>${p.price.toFixed(2)}</td>
                <td>${p.tp ? p.tp.toFixed(2) : '-'}</td>
                <td>${p.sl ? p.sl.toFixed(2) : '-'}</td>
                <td class="${pnlClass}">${p.pnl.toFixed(2)}</td>
            `;
            body.appendChild(tr);
        });
    }

    updatePerformanceSummary(totalPnl, null);
}

export function updateHistoryUI(history) {
    const body = document.getElementById('history-body');
    if (!body) return;
    body.innerHTML = '';
    
    let histPnl = 0;
    let wins = 0;
    
    if (history) {
        history.forEach(p => {
            histPnl += p.pnl;
            if (p.pnl > 0) wins++;
            
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td><span class="side-${p.side}">${p.side.toUpperCase()}</span></td>
                <td>${p.qty}</td>
                <td>${p.entry.toFixed(2)}</td>
                <td>${p.exit.toFixed(2)}</td>
                <td class="pnl-${p.pnl >= 0 ? 'up' : 'down'}">${p.pnl.toFixed(2)}</td>
            `;
            body.appendChild(tr);
        });
    }

    const winRate = history && history.length > 0 ? (wins / history.length * 100).toFixed(1) : 0;
    updatePerformanceSummary(null, { pnl: histPnl, wr: winRate });
}

function updatePerformanceSummary(activePnl, histData) {
    const pnlEl = document.getElementById('total-pnl');
    const wrEl = document.getElementById('win-rate');
    
    if (activePnl !== null) window._lastActivePnl = activePnl;
    if (histData !== null) window._lastHistData = histData;
    
    const totalPnl = (window._lastActivePnl || 0) + (window._lastHistData?.pnl || 0);
    const wr = window._lastHistData?.wr || 0;
    
    if (pnlEl) {
        pnlEl.textContent = `P/L: ${totalPnl.toFixed(2)}`;
        pnlEl.className = totalPnl >= 0 ? 'pnl-up' : 'pnl-down';
    }
    if (wrEl) wrEl.textContent = `WR: ${wr}%`;
}

export function setupPositionsPanel() {
    const tabActive = document.getElementById('tab-active');
    const tabHistory = document.getElementById('tab-history');
    
    if (tabActive) {
        tabActive.addEventListener('click', (e) => {
            e.stopPropagation();
            switchTradingTab('active');
        });
    }
    if (tabHistory) {
        tabHistory.addEventListener('click', (e) => {
            e.stopPropagation();
            switchTradingTab('history');
        });
    }
}

export function switchTradingTab(tab) {
    const activeTab = document.getElementById('tab-active');
    const histTab = document.getElementById('tab-history');
    const activeContent = document.getElementById('active-content');
    const histContent = document.getElementById('history-content');
    
    if (!activeTab || !histTab || !activeContent || !histContent) {
        console.warn("Positions UI elements missing for switchTradingTab");
        return;
    }
    
    if (tab === 'active') {
        activeTab.classList.add('active');
        histTab.classList.remove('active');
        activeContent.classList.remove('hidden');
        histContent.classList.add('hidden');
        showNotification("Viewing Active Positions", "info", 1200);
    } else {
        activeTab.classList.remove('active');
        histTab.classList.add('active');
        activeContent.classList.add('hidden');
        histContent.classList.remove('hidden');
        showNotification("Viewing Trade History", "info", 1200);
    }
}

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

window.showNotification = showNotification;
window.toggleLegend = toggleLegend;
window.addLegendItem = addLegendItem;
window.toggleTrendInfo = toggleTrendInfo;
window.scrollToRealTime = scrollToRealTime;
window.changeLayout = changeLayout;
window.createLayout = createLayout;
window.updatePositionsUI = updatePositionsUI;
window.hideLoader = hideLoader;
window.initCharts = initCharts;

// --- Modal Functions ---
window.showIndicatorSettings = function(indicatorName) {
    const meta = window.indicatorsMetadata.get(indicatorName);
    if (!meta || !meta.metadata) return;

    const overlay = document.getElementById('modal-overlay');
    const body = document.getElementById('modal-body');
    const nameEl = document.getElementById('modal-indicator-name');
    const saveBtn = document.getElementById('modal-save-btn');
    
    nameEl.textContent = `${indicatorName} Settings`;
    body.innerHTML = '';
    
    const currentParams = meta.params || {};
    
    Object.entries(meta.metadata).forEach(([key, schema]) => {
        const group = document.createElement('div');
        group.className = 'param-group';
        
        const label = document.createElement('label');
        label.textContent = key;
        
        const input = document.createElement('input');
        input.type = 'number';
        input.value = currentParams[key] !== undefined ? currentParams[key] : schema.default;
        if (schema.min !== undefined) input.min = schema.min;
        if (schema.max !== undefined) input.max = schema.max;
        if (schema.step !== undefined) input.step = schema.step;
        input.dataset.key = key;
        input.dataset.type = schema.type;
        
        group.appendChild(label);
        group.appendChild(input);
        body.appendChild(group);
    });
    
    saveBtn.onclick = () => {
        const newParams = {};
        body.querySelectorAll('input').forEach(input => {
            const val = input.dataset.type === 'int' ? parseInt(input.value) : parseFloat(input.value);
            newParams[input.dataset.key] = val;
        });
        
        window.CommandHandlers.update_indicator(indicatorName, newParams);
        window.closeModal();
    };
    
    overlay.classList.add('active');
};

window.closeModal = function() {
    const overlay = document.getElementById('modal-overlay');
    if (overlay) overlay.classList.remove('active');
};

// --- Initialization ---
document.addEventListener('DOMContentLoaded', () => {
    // Initialize Positions Panel Tabs
    if (typeof setupPositionsPanel === 'function') {
        setupPositionsPanel();
    }
    
    // Additional UI initializations can go here
    console.log("UI Components Initialized.");
});
