/**
 * Core Charting & Sync Logic
 */

// Global state
window.charts = new Map();       // chartId -> chartInstance
window.seriesMap = new Map();    // seriesId -> seriesInstance
window.chartSeriesMap = new Map(); // chartId -> Set<seriesId>
window.chartTimeZone = 'Asia/Kolkata';
window.tooltipEnabled = false;

// Disable context menu (right-click) on the window
window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
}, false);

// Error Handling
window.onerror = function (msg, url, line, col, error) {
    const loadingDiv = document.getElementById('loading');
    if (loadingDiv) {
        loadingDiv.innerHTML += `<div style="color:red; font-size:16px; margin-top:10px;">Error: ${msg}<br>Line: ${line}:${col}</div>`;
    }
    console.error("Global Error:", msg, url, line, col, error);

    // Bridge to Rust->Python
    if (window.__TAURI__) {
        try {
            const invoke = window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke;
            if (invoke) {
                invoke('emit_to_backend', { 
                    action: 'js_error', 
                    data: { 
                        message: msg, 
                        url: url, 
                        line: line, 
                        col: col, 
                        stack: error ? error.stack : "" 
                    } 
                }).catch(e => console.error("Error reporting failed:", e));
            }
        } catch (e) {
            // Silently fail to avoid infinite recursion if bridge itself crashes
        }
    }
    return false;
};

window.onunhandledrejection = function(event) {
    console.error("Unhandled Rejection:", event.reason);
    if (window.__TAURI__) {
        try {
            const invoke = window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke;
            if (invoke) {
                invoke('emit_to_backend', { 
                    action: 'js_error', 
                    data: { 
                        message: `Unhandled Rejection: ${event.reason}`,
                        stack: event.reason && event.reason.stack ? event.reason.stack : ""
                    } 
                }).catch(e => console.error("Error reporting failed:", e));
            }
        } catch (e) {}
    }
};

function updateStatus(text) {
    const el = document.getElementById('loading');
    if (el) el.innerText = text;
}

// Global Help: find a series for a chart (for plugins)
window.getSeriesForChart = function (chartId) {
    const seriesSet = window.chartSeriesMap.get(chartId);
    if (seriesSet && seriesSet.size > 0) {
        // Find the first series that actually exists in our global map
        for (const sid of seriesSet) {
            const series = window.seriesMap.get(sid);
            if (series) return series;
        }
    }
    return null;
};

// --- Sync Manager ---
export const SyncManager = {
    enabled: false,
    isSyncing: false,
    charts: [],
    activeChart: null,
    _rafId: null,
    lastMaster: null,
    lastHigh: null,
    lastLow: null,

    register: function (chart, container) {
        if (this.charts.includes(chart)) return;
        this.charts.push(chart);

        chart.timeScale().subscribeVisibleLogicalRangeChange((range) => {
            if (!this.enabled || this.isSyncing || !range) return;
            this.isSyncing = true;
            try {
                this.charts.forEach(otherChart => {
                    // Check if otherChart is still in the DOM and visible
                    const chartId = [...window.charts.entries()].find(([_, c]) => c === otherChart)?.[0];
                    const cell = document.getElementById(chartId?.replace('chart-', 'chart-cell-'));
                    if (otherChart !== chart && cell && cell.style.display !== 'none') {
                        otherChart.timeScale().setVisibleLogicalRange(range);
                    }
                });
            } finally { this.isSyncing = false; }
            this.triggerRedraw();
        });

        if (container) {
            container.addEventListener('mouseenter', () => { this.activeChart = chart; });
        }

        if (!this._rafId) this.startLoop();

        chart.subscribeCrosshairMove((param) => {
            if (!this.enabled || this.isSyncing) return;
            this.isSyncing = true;
            try {
                this.charts.forEach(otherChart => {
                    if (otherChart === chart) return;
                    
                    const targetChartId = [...window.charts.entries()].find(([_, c]) => c === otherChart)?.[0];
                    if (!targetChartId) return;

                    const cell = document.getElementById(targetChartId.replace('chart-', 'chart-cell-'));
                    if (!cell || cell.style.display === 'none') return;

                    if (!param.time || param.point === undefined || param.point.x < 0) {
                        otherChart.clearCrosshairPosition();
                    } else {
                        const series = window.getSeriesForChart(targetChartId);
                        if (series) otherChart.setCrosshairPosition(NaN, param.time, series);
                    }
                });
            } finally { this.isSyncing = false; }
            this.triggerRedraw();
        });
    },

    startLoop: function () {
        // Optimization: Removed constant polling loop.
        // Redraw only when necessary via registered listeners on crosshair and visible range.
        this.triggerRedraw = () => {
            if (this._rafId) return;
            this._rafId = requestAnimationFrame(() => {
                this.syncScales();
                this._rafId = null;
            });
        };
    },

    syncScales: function () {
        if (!this.enabled || this.isSyncing) return;
        const master = this.activeChart;
        if (!master) return;

        const ps = master.priceScale('right');
        const height = master.options().height || 0;
        let targetLow = null, targetHigh = null;

        if (height > 0 && ps && typeof ps.coordinateToPrice === 'function') {
            const hStart = ps.coordinateToPrice(1);
            const hEnd = ps.coordinateToPrice(height - 1);
            if (hStart !== null && hEnd !== null) { targetHigh = hStart; targetLow = hEnd; }
        }

        if (targetHigh === null || targetLow === null || (this.lastMaster === master && this.lastHigh === targetHigh && this.lastLow === targetLow)) return;

        this.lastMaster = master; this.lastHigh = targetHigh; this.lastLow = targetLow;
        this.isSyncing = true;
        try {
            this.charts.forEach(otherChart => {
                if (otherChart === master) return;
                let targetChartId = [...window.charts.entries()].find(([id, c]) => c === otherChart)?.[0];
                if (targetChartId) {
                    const seriesSet = window.chartSeriesMap.get(targetChartId);
                    if (seriesSet) {
                        seriesSet.forEach(sid => {
                            const s = window.seriesMap.get(sid);
                            if (s) s.applyOptions({ autoscaleInfoProvider: () => ({ priceRange: { minValue: targetLow, maxValue: targetHigh } }) });
                        });
                        otherChart.priceScale('right').applyOptions({ autoScale: true });
                    }
                }
            });
        } finally { this.isSyncing = false; }
    }
};

window.SyncManager = SyncManager;

// --- Window Resize ---
window.addEventListener('resize', () => {
    window.charts.forEach((chart, id) => {
        const cell = document.getElementById(id.replace('chart-', 'chart-cell-'));
        if (cell && cell.style.display !== 'none') {
            chart.applyOptions({ width: cell.clientWidth, height: cell.clientHeight });
        }
    });
});

window.bridgeLog = function (msg) {
    console.log("Bridge:", msg);
};
