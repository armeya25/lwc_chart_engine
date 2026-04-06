/**
 * Core Charting & Sync Logic
 */

// Global state
window.charts = new Map();       // chartId -> chartInstance
window.seriesMap = new Map();    // seriesId -> seriesInstance
window.chartSeriesMap = new Map(); // chartId -> Set<seriesId>
window.chartTimeZone = 'Asia/Kolkata';
window.tooltipEnabled = true;

// Error Handling
window.onerror = function (msg, url, line, col, error) {
    const loadingDiv = document.getElementById('loading');
    if (loadingDiv) {
        loadingDiv.innerHTML += `<div style="color:red; font-size:16px; margin-top:10px;">Error: ${msg}<br>Line: ${line}:${col}</div>`;
    }
    console.error("Global Error:", msg, url, line, col, error);
    return false;
};

function updateStatus(text) {
    const el = document.getElementById('loading');
    if (el) el.innerText = text;
}

// Global Help: find a series for a chart (for plugins)
window.getSeriesForChart = function (chartId) {
    const seriesSet = window.chartSeriesMap.get(chartId);
    if (seriesSet && seriesSet.size > 0) {
        const firstId = seriesSet.values().next().value;
        return window.seriesMap.get(firstId);
    }
    return window.seriesMap.get('main');
};

// --- Sync Manager ---
const SyncManager = {
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
                    if (otherChart !== chart) otherChart.timeScale().setVisibleLogicalRange(range);
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
                    if (!param.time || param.point === undefined || param.point.x < 0) {
                        otherChart.clearCrosshairPosition(); return;
                    }
                    let targetChartId = [...window.charts.entries()].find(([id, c]) => c === otherChart)?.[0];
                    if (targetChartId) {
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
