/**
 * Command Dispatcher & Handlers
 */
import { SyncManager } from './core.js';
import { showNotification, createLayout, hideLoader } from './ui.js';

export const CommandQueue = {
    queue: [],
    isProcessing: false,
    BUDGET_MS: 8,

    push: function (cmd) {
        this.queue.push(cmd);
        if (!this.isProcessing) {
            this.isProcessing = true;
            requestAnimationFrame(() => this.process());
        }
    },

    process: function () {
        const start = performance.now();
        while (this.queue.length > 0) {
            if (performance.now() - start > this.BUDGET_MS) {
                requestAnimationFrame(() => this.process()); return;
            }
            
            // Peak at the next command instead of shifting immediately
            const cmd = this.queue[0];
            const wasProcessed = this.processCommandSync(cmd);
            
            if (wasProcessed) {
                this.queue.shift(); // Only remove if successfully processed
            } else {
                // If not ready, wait for the next frame
                requestAnimationFrame(() => this.process());
                return;
            }
        }
        this.isProcessing = false;
    },

    processCommandSync: function (cmd) {
        const { action, chartId = 'chart-0' } = cmd;
        const handler = CommandHandlers[action];
        
        if (!handler) {
            console.warn("Unknown command action:", action);
            return true; // Mark as processed to remove from queue
        }

        try {
            const targetChart = window.charts.get(chartId);
            
            // Core safety: If the target chart is missing and it's not a global command, 
            // signal that we aren't ready to process this yet.
            const isGlobal = (action === 'set_layout' || action === 'show_notification' || action === 'hide_loading');
            if (!targetChart && !isGlobal) {
                return false; 
            }

            handler(targetChart, cmd, chartId);
            return true;
        } catch (e) {
            console.error(`Error executing ${action} on ${chartId}:`, e);
            const status = document.querySelector('.loading-text');
            if (status) status.innerText = `Error: ${action}`;
        }
    }
};

const getSId = (cmd) => cmd.id || cmd.seriesId || cmd.series_id;

window.isReady = false;
window.indicatorsMetadata = new Map(); // Store { params, metadata, chartId, type }

export function handleCommand(cmd) {
    if (typeof cmd === 'string') cmd = JSON.parse(cmd);
    CommandQueue.push(cmd);
};

// --- Helper for Multiple Indicator Panes ---
function ensurePaneLayout(chart, targetPaneId = null) {
    if (!chart) return;

    // 1. Identify which chart this is (find chartId)
    let chartId = null;
    for (let [id, c] of window.charts.entries()) {
        if (c === chart) { chartId = id; break; }
    }
    
    // Fallback search if chartId is not immediately resolved
    if (!chartId) {
        chartId = window.currentChartId || Array.from(window.charts.keys())[0];
    }
    if (!chartId) return;

    // 2. Collect visible panes SPECIFIC to this chart
    const seriesOnChart = window.chartSeriesMap.get(chartId) || new Set();
    const visiblePanes = new Set();
    const allPanesOnChart = new Set();

    seriesOnChart.forEach(sid => {
        const series = window.seriesMap.get(sid);
        if (series) {
            const options = series.options();
            const pid = options.priceScaleId;
            if (pid && pid.startsWith('pane_')) {
                allPanesOnChart.add(pid);
                if (options.visible !== false) visiblePanes.add(pid);
            }
        }
    });
    
    // If we are currently adding a new series, its pane should be counted
    if (targetPaneId) {
        visiblePanes.add(targetPaneId);
        allPanesOnChart.add(targetPaneId);
    }

    const numPanes = visiblePanes.size;
    
    // 3. Rebalance layout
    if (numPanes === 0) {
        chart.priceScale('right').applyOptions({
            scaleMargins: { top: 0.1, bottom: 0.1 }
        });
        // Zero-out any orphaned panes on this chart
        allPanesOnChart.forEach(pid => {
            chart.priceScale(pid).applyOptions({ scaleMargins: { top: 2, bottom: 0 } });
        });
        return;
    }

    const paneHeight = 0.20; 
    const totalIndicatorHeight = Math.min(0.5, numPanes * paneHeight);
    const priceHeight = 1.0 - totalIndicatorHeight;

    chart.priceScale('right').applyOptions({
        scaleMargins: { top: 0.05, bottom: totalIndicatorHeight + 0.02 }
    });

    const paneList = Array.from(visiblePanes).sort();
    paneList.forEach((pid, idx) => {
        const bottomOffset = (numPanes - 1 - idx) * (totalIndicatorHeight / numPanes);
        chart.priceScale(pid).applyOptions({
            scaleMargins: { 
                top: priceHeight + (idx * (totalIndicatorHeight / numPanes)) + 0.02, 
                bottom: bottomOffset 
            },
            borderVisible: false
        });
    });

    // Ensure hidden panes don't take space
    allPanesOnChart.forEach(pid => {
        if (!visiblePanes.has(pid)) {
            chart.priceScale(pid).applyOptions({ scaleMargins: { top: 2, bottom: 0 } });
        }
    });
}

export const CommandHandlers = {
    ensurePaneLayout: (chart, targetPaneId) => ensurePaneLayout(chart, targetPaneId),
    configure_chart: (targetChart, cmd) => { if (targetChart) targetChart.applyOptions(cmd.data); },
    set_layout: (_targetChart, cmd) => { 
        const type = cmd.layout || cmd.data?.type || 'single';
        createLayout(type); 
        if (hideLoader) hideLoader();
    },
    create_line_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;

        // Handle Pane Scaling for indicators
        if (cmd.options && cmd.options.priceScaleId && cmd.options.priceScaleId.startsWith('pane_')) {
            ensurePaneLayout(targetChart, cmd.options.priceScaleId);
        }

        const series = targetChart.addSeries(LightweightCharts.LineSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        
        const extra = cmd.extra || {};
        const indicatorId = cmd.indicator || extra.indicator;
        
        if (indicatorId) {
            window.indicatorsMetadata.set(indicatorId, {
                sid, 
                chartId, 
                params: extra.indicatorParams || cmd.indicatorParams || {}, 
                metadata: extra.indicatorMetadata || cmd.indicatorMetadata || {}, 
                ownerId: extra.owner_id || cmd.owner_id,
                indType: extra.ind_type || cmd.ind_type,
                type: 'line'
            });
        }
        const humanName = extra.humanName || extra.human_name || cmd.humanName || cmd.name;
        const indicatorTypeName = extra.indicatorTypeName || extra.indicator_type_name || cmd.indicatorTypeName;
        window.addLegendItem(chartId, sid, cmd.name, cmd.options.color, 'line', indicatorId, humanName, indicatorTypeName);
        console.info(`[Chart] Created Indicator Series: ${sid} for ${indicatorId || 'Overlay'}`);
    },
    create_area_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;

        // Handle Pane Scaling for indicators
        if (cmd.options && cmd.options.priceScaleId && cmd.options.priceScaleId.startsWith('pane_')) {
            ensurePaneLayout(targetChart, cmd.options.priceScaleId);
        }

        const series = targetChart.addSeries(LightweightCharts.AreaSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        const humanName = cmd.humanName || (cmd.extra ? (cmd.extra.humanName || cmd.extra.human_name) : cmd.name);
        const indicatorTypeName = cmd.indicatorTypeName || (cmd.extra ? (cmd.extra.indicatorTypeName || cmd.extra.indicator_type_name) : null);
        window.addLegendItem(chartId, sid, cmd.name, cmd.options.lineColor || cmd.options.topColor, 'area', cmd.indicator, humanName, indicatorTypeName);
    },
    create_segmented_line: (_targetChart, cmd) => {
        const ownerSeries = window.seriesMap.get(getSId(cmd));
        if (ownerSeries && window.SegmentedLinePrimitive) {
            if (!ownerSeries._segmentedPlugin) {
                const plugin = new window.SegmentedLinePrimitive(cmd.options || {});
                ownerSeries.attachPrimitive(plugin);
                ownerSeries._segmentedPlugin = plugin;
                // Keep series visible but hide its main line so coordinates are calculated
                ownerSeries.applyOptions({ 
                    lineWidth: 0,
                    priceLineVisible: false,
                    lastValueVisible: false,
                    crosshairMarkerVisible: false
                });
            }
            ownerSeries._segmentedPlugin.setData(cmd.data);
        }
    },
    create_segmented_band: (_targetChart, cmd) => {
        const ownerSeries = window.seriesMap.get(getSId(cmd));
        if (ownerSeries && window.SegmentedBandPrimitive) {
            if (!ownerSeries._segmentedBandPlugin) {
                const plugin = new window.SegmentedBandPrimitive(cmd.options || {});
                ownerSeries.attachPrimitive(plugin);
                ownerSeries._segmentedBandPlugin = plugin;
                // Hide main representative series
                ownerSeries.applyOptions({ 
                    lineWidth: 0,
                    priceLineVisible: false,
                    lastValueVisible: false,
                    lineVisible: false,
                    areaVisible: false
                });
            }
            ownerSeries._segmentedBandPlugin.setData(cmd.data);
        }
    },
    create_candlestick_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;
        const series = targetChart.addSeries(LightweightCharts.CandlestickSeries, cmd.options);
        series._name = cmd.name;
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        const humanName = cmd.humanName || (cmd.extra ? (cmd.extra.humanName || cmd.extra.human_name) : cmd.name);
        const indicatorTypeName = cmd.indicatorTypeName || (cmd.extra ? (cmd.extra.indicatorTypeName || cmd.extra.indicator_type_name) : null);
        if (cmd.indicator) {
            const extra = cmd.extra || {};
            window.indicatorsMetadata.set(cmd.indicator, {
                sid, 
                chartId, 
                params: extra.indicatorParams || cmd.indicatorParams || {}, 
                metadata: extra.indicatorMetadata || cmd.indicatorMetadata || {}, 
                ownerId: extra.owner_id || cmd.owner_id,
                indType: extra.ind_type || cmd.ind_type,
                type: 'candle'
            });
        }
        window.addLegendItem(chartId, sid, cmd.name, cmd.options.upColor, 'candle', cmd.indicator, humanName, indicatorTypeName);
    },
    create_histogram_series: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const sid = getSId(cmd);
        if (window.seriesMap.has(sid)) return;

        // Handle Pane Scaling for indicators
        if (cmd.options && cmd.options.priceScaleId && cmd.options.priceScaleId.startsWith('pane_')) {
            ensurePaneLayout(targetChart, cmd.options.priceScaleId);
        }

        const series = targetChart.addSeries(LightweightCharts.HistogramSeries, cmd.options);
        window.seriesMap.set(sid, series);
        if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
        window.chartSeriesMap.get(chartId).add(sid);
        const humanName = cmd.humanName || (cmd.extra ? (cmd.extra.humanName || cmd.extra.human_name) : cmd.name);
        const indicatorTypeName = cmd.indicatorTypeName || (cmd.extra ? (cmd.extra.indicatorTypeName || cmd.extra.indicator_type_name) : null);
        if (cmd.indicator) {
            const extra = cmd.extra || {};
            window.indicatorsMetadata.set(cmd.indicator, {
                sid, 
                chartId, 
                params: extra.indicatorParams || cmd.indicatorParams || {}, 
                metadata: extra.indicatorMetadata || cmd.indicatorMetadata || {}, 
                ownerId: extra.owner_id || cmd.owner_id,
                indType: extra.ind_type || cmd.ind_type,
                type: 'histogram'
            });
        }
        window.addLegendItem(chartId, sid, cmd.name, cmd.options.color, 'histogram', cmd.indicator, humanName, indicatorTypeName);
    },
    set_series_data: (_targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series) {
            // Update legend metadata if provided
            const humanName = cmd.humanName || (cmd.extra ? (cmd.extra.humanName || cmd.extra.human_name) : null);
            const indicatorTypeName = cmd.indicatorTypeName || (cmd.extra ? (cmd.extra.indicatorTypeName || cmd.extra.indicator_type_name) : (cmd.indicatorType || (cmd.extra ? cmd.extra.indicator_type : null)));
            if (cmd.indicator || humanName || indicatorTypeName) {
                // Determine chartId for this series
                let chartIdToUse = null;
                for (let [id, sSet] of window.chartSeriesMap.entries()) {
                    if (sSet.has(sid)) { chartIdToUse = id; break; }
                }
                if (chartIdToUse) {
                    window.addLegendItem(chartIdToUse, sid, cmd.name || sid, null, null, cmd.indicator, humanName, indicatorTypeName);
                }
            }

            // If this series has a segmented plugin, update it too
            if (series._segmentedPlugin) {
                series._segmentedPlugin.setData(cmd.data);
            }
            
            // Auto-fallback: if it's a line/area series and data has no 'value', use 'close'
            const processedData = (cmd.data || []).map(item => {
                let val = item.value;
                if (val === undefined && item.close !== undefined) val = item.close;
                return { ...item, value: val };
            }).filter(item => item.value !== null && !isNaN(item.value));

            series.setData(processedData);
            
            // Aggressive Rebalance: If this is an indicator pane, force a layout check
            const pid = series.options().priceScaleId;
            if (pid && pid.startsWith('pane_')) {
                ensurePaneLayout(_targetChart || window.charts.values().next().value);
            }

            if (_targetChart && cmd.data && cmd.data.length > 5) _targetChart.timeScale().fitContent();
            if (typeof BoxManager !== 'undefined') BoxManager.updatePositions();
            if (typeof PositionToolManager !== 'undefined') PositionToolManager.updatePositions();
            if (typeof LineToolManager !== 'undefined') LineToolManager.updatePositions();
        } else {
            console.warn(`[JS] set_series_data: Series NOT found: ${sid}`);
        }
    },
    set_volume_data: (targetChart, cmd, chartId) => {
        if (!targetChart) return;
        const mainSid = getSId(cmd);
        const volumeSid = `volume-${mainSid}`;
        
        let volumeSeries = window.seriesMap.get(volumeSid);
        if (!volumeSeries) {
            volumeSeries = targetChart.addSeries(LightweightCharts.HistogramSeries, {
                color: '#26a69a',
                priceFormat: { type: 'volume' },
                priceScaleId: 'volume', // Dedicated scale for volume pane
            });
            
            // Configure the volume scale to be at the bottom
            targetChart.priceScale('volume').applyOptions({
                scaleMargins: {
                    top: 0.8, // Reserve top 80% for Price
                    bottom: 0,
                },
            });
            
            window.seriesMap.set(volumeSid, volumeSeries);
            if (!window.chartSeriesMap.has(chartId)) window.chartSeriesMap.set(chartId, new Set());
            window.chartSeriesMap.get(chartId).add(volumeSid);
            window.addLegendItem(chartId, volumeSid, 'Volume', '#26a69a', 'histogram');
        }

        const volumeData = cmd.data.map(d => ({
            time: d.time,
            value: d.volume,
            color: d.close >= d.open ? 'rgba(38, 166, 154, 0.5)' : 'rgba(239, 83, 80, 0.5)'
        }));
        volumeSeries.setData(volumeData);
    },
    update_volume_data: (targetChart, cmd) => {
        if (!targetChart) return;
        const mainSid = getSId(cmd);
        const volumeSid = `volume-${mainSid}`;
        const volumeSeries = window.seriesMap.get(volumeSid);
        if (volumeSeries && cmd.data) {
            const d = cmd.data;
            volumeSeries.update({
                time: d.time,
                value: d.volume,
                color: d.close >= d.open ? 'rgba(38, 166, 154, 0.5)' : 'rgba(239, 83, 80, 0.5)'
            });
        }
    },
    update_series_data: (_targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series) {
            // Update legend metadata if provided
            const humanName = cmd.humanName || (cmd.extra ? (cmd.extra.humanName || cmd.extra.human_name) : null);
            const indicatorTypeName = cmd.indicatorTypeName || (cmd.extra ? (cmd.extra.indicatorTypeName || cmd.extra.indicator_type_name) : (cmd.indicatorType || (cmd.extra ? cmd.extra.indicator_type : null)));
            if (cmd.indicator || humanName || indicatorTypeName) {
                let chartIdToUse = null;
                for (let [id, sSet] of window.chartSeriesMap.entries()) {
                    if (sSet.has(sid)) { chartIdToUse = id; break; }
                }
                if (chartIdToUse) {
                    window.addLegendItem(chartIdToUse, sid, cmd.name || sid, null, null, cmd.indicator, humanName, indicatorTypeName);
                }
            }
            try {
                series.update(cmd.data);
            } catch (e) {
                if (!e.message.includes('oldest data')) {
                    console.error(`Update failed for ${cmd.id}:`, e.message);
                    throw e;
                }
            }
        }
    },
    update_series_options: (_targetChart, cmd) => {
        const series = window.seriesMap.get(getSId(cmd));
        if (series) series.applyOptions(cmd.options);
    },
    remove_series: (targetChart, cmd) => {
        const sid = getSId(cmd);
        const series = window.seriesMap.get(sid);
        if (series && targetChart) {
            targetChart.removeSeries(series);
            window.seriesMap.delete(sid);
            // Remove legend item from the correct chart legend
            let chartIdForSeries = null;
            window.chartSeriesMap.forEach((sSet, cid) => {
                if (sSet.has(sid)) { chartIdForSeries = cid; sSet.delete(sid); }
            });
            const item = document.getElementById(`${chartIdForSeries || ''}-legend-item-${sid}`);
            if (item) item.remove();
        }
    },
    clear_all_series: (targetChart, _cmd, chartId) => {
        if (!targetChart) return;
        const sSet = window.chartSeriesMap.get(chartId);
        if (sSet) {
            sSet.forEach(sid => {
                const series = window.seriesMap.get(sid);
                if (series) targetChart.removeSeries(series);
                window.seriesMap.delete(sid);
                const item = document.getElementById(`${chartId}-legend-item-${sid}`);
                if (item) item.remove();
            });
            sSet.clear();
        }
    },
    fit_content: (targetChart) => { if (targetChart) targetChart.timeScale().fitContent(); },
    set_visible_range: (targetChart, cmd) => { if (targetChart) targetChart.timeScale().setVisibleRange(cmd.data); },
    create_position: (_targetChart, cmd, chartId) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.create(chartId, cmd.id, cmd.data); },
    update_position: (_targetChart, cmd) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.update(cmd.id, cmd.data); },
    remove_position: (_targetChart, cmd) => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.remove(cmd.id); },
    create_box: (_targetChart, cmd, chartId) => { 
        if (typeof BoxManager !== 'undefined') {
            BoxManager.createBox(chartId, cmd.id, cmd.data); 
        }
    },
    update_box: (_targetChart, cmd) => { if (typeof BoxManager !== 'undefined') BoxManager.updateBox(cmd.id, cmd.data); },
    remove_box: (_targetChart, cmd) => { if (typeof BoxManager !== 'undefined') BoxManager.removeBox(cmd.id); },
    create_line_tool: (_targetChart, cmd, chartId) => { if (typeof LineToolManager !== 'undefined') LineToolManager.create(chartId, cmd.id, cmd.data); },
    update_line_tool: (_targetChart, cmd) => { if (typeof LineToolManager !== 'undefined') LineToolManager.update(cmd.id, cmd.data); },
    remove_line_tool: (_targetChart, cmd) => { if (typeof LineToolManager !== 'undefined') LineToolManager.remove(cmd.id); },
    add_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.addMarker(getSId(cmd), cmd.data, chartId); },
    add_markers_bulk: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.addMarkersBulk(getSId(cmd), cmd.data, chartId); },
    remove_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.removeMarker(getSId(cmd), cmd.marker_id, chartId); },
    update_marker: (_targetChart, cmd, chartId) => { if (typeof MarkerManager !== 'undefined') MarkerManager.updateMarker(getSId(cmd), cmd.marker_id, cmd.data, chartId); },
    remove_all_markers: () => { if (typeof MarkerManager !== 'undefined') MarkerManager.clearAll(); },
    remove_all_positions: () => { if (typeof PositionToolManager !== 'undefined') PositionToolManager.removeAll(); },
    remove_all_boxes: () => { if (typeof BoxManager !== 'undefined') BoxManager.removeAll(); },
    remove_all_line_tools: () => { if (typeof LineToolManager !== 'undefined') LineToolManager.removeAll(); },
    create_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.create(getSId(cmd), cmd.line_id, cmd.options); },
    remove_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.remove(cmd.line_id); },
    update_price_line: (_targetChart, cmd) => { if (typeof PriceLineManager !== 'undefined') PriceLineManager.update(cmd.line_id, cmd.options); },
    set_watermark: (targetChart, cmd) => {
        if (!targetChart) return;
        const d = cmd.data || {};
        if (typeof LightweightCharts.createTextWatermark === 'function') {
            const pane = targetChart.panes()[0];
            if (pane) LightweightCharts.createTextWatermark(pane, { 
                horzAlign: d.horzAlign || 'center', 
                vertAlign: d.vertAlign || 'center', 
                lines: [{ text: d.text, color: d.color || 'rgba(255, 255, 255, 0.1)', fontSize: d.fontSize || 48, fontWeight: 'bold' }] 
            });
        } else {
            targetChart.applyOptions({ 
                watermark: { 
                    visible: true, 
                    text: d.text, 
                    color: d.color || 'rgba(255, 255, 255, 0.1)', 
                    horzAlign: d.horzAlign || 'center', 
                    vertAlign: d.vertAlign || 'center', 
                    fontSize: d.fontSize || 48 
                } 
            });
        }
    },
    set_tooltip: (_targetChart, cmd) => {
        window.tooltipEnabled = !!cmd.data.enabled;
        if (!window.tooltipEnabled) document.querySelectorAll('.floating-tooltip').forEach(el => el.style.opacity = '0');
    },
    configure_price_scale: (targetChart, cmd) => {
        if (!targetChart) return;
        const d = cmd.data, scaleId = d.scaleId || 'right';
        targetChart.priceScale(scaleId).applyOptions({ visible: true, mode: d.mode !== undefined ? d.mode : 0, autoScale: d.autoScale !== undefined ? d.autoScale : true, invertScale: d.invertScale || false, scaleMargins: d.scaleMargins || { top: 0.05, bottom: 0.05 } });
    },
    set_sync: (_targetChart, cmd) => {
        SyncManager.enabled = !!cmd.data.enabled;
        if (!SyncManager.enabled) {
            if (SyncManager._rafId) { cancelAnimationFrame(SyncManager._rafId); SyncManager._rafId = null; }
            window.charts.forEach(c => c.clearCrosshairPosition());
        } else {
            window.charts.forEach((c, id) => SyncManager.register(c, document.getElementById(id.replace('chart-', 'chart-cell-'))));
        }
    },
    set_crosshair_mode: (targetChart, cmd) => { if (targetChart) targetChart.applyOptions({ crosshair: { mode: cmd.data.mode } }); },
    set_timezone: (_targetChart, cmd) => {
        window.chartTimeZone = cmd.data.timezone || 'UTC';
        const localization = { timeFormatter: (ts) => typeof ts !== 'number' ? String(ts) : new Date(ts * 1000).toLocaleString('en-GB', { timeZone: window.chartTimeZone, day: 'numeric', month: 'short', year: '2-digit', hour: '2-digit', minute: '2-digit', hour12: false }).replace(',', '') };
        const timeScale = { tickMarkFormatter: (time) => { const d = typeof time === 'number' ? new Date(time * 1000) : new Date(time); return isNaN(d) ? "" : d.toLocaleDateString('en-GB', { month: 'short', day: 'numeric', timeZone: window.chartTimeZone }); } };
        window.charts.forEach(c => c.applyOptions({ localization, timeScale }));
    },
    hide_loading: () => {
        if (window.hideLoader) window.hideLoader();
    },
    show_notification: (_targetChart, cmd) => {
        // Ensure notification container exists and has correct styling
        let container = document.getElementById('notification-container');
        if (!container) {
            container = document.createElement('div');
            container.id = 'notification-container';
            container.style.cssText = `position: fixed; bottom: 20px; left: 20px; z-index: 10000; display: flex; flex-direction: column-reverse; gap: 10px; pointer-events: none;`;
            document.body.appendChild(container);
        }
        showNotification(cmd.data.message, cmd.data.type || 'info', cmd.data.duration || 3000, cmd.data.text_color || null);
    },
    set_info_panel_visibility: (_targetChart, cmd) => {
        const panel = document.getElementById('info-panel');
        if (panel) panel.classList.toggle('hidden', !cmd.data.visible);
    },
    set_legend_visibility: (_targetChart, cmd) => {
        const visible = cmd.data.visible;
        document.querySelectorAll('.chart-legend').forEach(el => {
            el.classList.toggle('hidden', !visible);
        });
    },
    update_positions: (_targetChart, cmd) => {
        if (window.updatePositionsUI) window.updatePositionsUI(cmd.data);
    },
    update_history: (_targetChart, cmd) => {
        if (window.updateHistoryUI) window.updateHistoryUI(cmd.data);
    },
    set_trading_visibility: (_targetChart, cmd) => {
        const trade = document.getElementById('trade-panel');
        const pos = document.getElementById('positions-panel');
        const visible = cmd.data.visible;
        if (trade) {
            trade.classList.toggle('hidden', !visible);
        }
        if (pos) {
            pos.classList.toggle('hidden', !visible);
            window.positionsUserHidden = !visible;
        }
    },
    update_info_panel: (_targetChart, cmd) => {
        const d = cmd.data || {};
        const content = document.getElementById('info-content');
        if (!content) return;

        Object.entries(d).forEach(([key, val]) => {
            const elId = `info-${key.toLowerCase().replace(/\s+/g, '-')}`;
            let row = document.getElementById(elId);
            if (!row) {
                row = document.createElement('div');
                row.className = 'info-row';
                row.id = elId;
                row.innerHTML = `<span>${key}</span><span class="info-val">NA</span>`;
                content.appendChild(row);
            }
            const valEl = row.querySelector('.info-val');
            if (valEl) {
                valEl.textContent = val || 'NA';
                
                // Color coding based on value
                const lowerVal = String(val).toLowerCase();
                valEl.classList.remove('info-up', 'info-down', 'info-neutral');
                if (lowerVal.includes('up') || lowerVal.includes('bull') || lowerVal.includes('buy')) valEl.classList.add('info-up');
                else if (lowerVal.includes('down') || lowerVal.includes('bear') || lowerVal.includes('sell')) valEl.classList.add('info-down');
                else if (lowerVal.includes('neutral') || lowerVal.includes('sideway')) valEl.classList.add('info-neutral');
            }
        });

        // Ensure panel is visible when data arrives
        const panel = document.getElementById('info-panel');
        if (panel) {
            panel.classList.remove('hidden');
            panel.classList.remove('collapsed');
        }
    },
    take_screenshot: (targetChart, cmd, chartId) => {
        if (targetChart) {
            const canvas = targetChart.takeScreenshot();
            const base64 = canvas.toDataURL('image/png');
            // Emit to backend (Tauri or PyWebview)
            const invoke = window.__TAURI__ ? (window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke) : null;
            if (invoke) {
                invoke('emit_to_backend', { 
                    action: 'screenshot', 
                    data: { 
                        base64, 
                        filename: cmd.filename || `chart_${chartId}_${new Date().getTime()}.png`,
                        chartId 
                    } 
                });
            }
        }
    },
    remove_indicator: (chart, arg, optChartId) => {
        let indicatorName = null;
        let chartId = optChartId;

        if (typeof chart === 'string') {
            // Called from UI: remove_indicator(name, chartId)
            indicatorName = chart;
            chartId = arg; 
        } else {
            // Called from Dispatcher: remove_indicator(chart, cmd/name, chartId)
            indicatorName = typeof arg === 'string' ? arg : (arg ? arg.indicator : null);
        }
        
        if (!indicatorName || !chartId) return;

        const group = document.getElementById(`${chartId}-legend-group-${indicatorName}`);
        if (group) {
            const sids = new Set();
            
            // 1. Collect main series ID
            if (group.dataset.mainSid) sids.add(group.dataset.mainSid);
            
            // 2. Collect all sub-item series IDs
            const groupContent = group.querySelector('.legend-group-content');
            if (groupContent) {
                groupContent.querySelectorAll('.legend-sub-item').forEach(item => {
                    if (item.dataset.seriesId) sids.add(item.dataset.seriesId);
                });
            }
            
            // 3. Remove all series from all charts
            sids.forEach(sid => {
                window.charts.forEach(chart => {
                    const series = window.seriesMap.get(sid);
                    if (series) {
                        try { chart.removeSeries(series); } catch(e) {}
                    }
                });
                window.seriesMap.delete(sid);
                window.chartSeriesMap.forEach(sSet => sSet.delete(sid));
            });

            // 4. Notify Backend
            const invoke = window.__TAURI__ ? (window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke) : null;
            if (invoke) {
                invoke('emit_to_backend', { 
                    action: 'remove_indicator', 
                    data: { indicator: indicatorName } 
                });
            }

            if (group) {
                group.remove();
                // 5. Re-balance layout for all charts to reclaim space
                window.charts.forEach(chart => {
                    if (window.CommandHandlers.ensurePaneLayout) {
                        window.CommandHandlers.ensurePaneLayout(chart);
                    }
                });
            }
        }
    },
    open_indicator_settings: (chart, arg) => {
        let indicatorName = null;
        if (typeof chart === 'string') {
            // Called from UI: open_indicator_settings(name)
            indicatorName = chart;
        } else {
            // Called from Dispatcher: open_indicator_settings(chart, cmd/name)
            indicatorName = typeof arg === 'string' ? arg : (arg ? arg.indicator : null);
        }

        if (indicatorName && window.showIndicatorSettings) {
            window.showIndicatorSettings(indicatorName);
        }
    },
    register_indicator_metadata: (_targetChart, cmd) => {
        const indicator = cmd.indicator;
        const data = cmd.data;
        window.indicatorsMetadata.set(indicator, {
            params: data.params,
            metadata: data.schema,
            ownerId: data.owner_id,
            indType: data.ind_type
        });
    },
    update_indicator: (_targetChart, arg1, arg2) => {
        let indicatorName = null;
        let newParams = null;

        if (typeof arg1 === 'string') {
            // Called from UI: update_indicator(name, params)
            indicatorName = arg1;
            newParams = arg2;
        } else if (arg1 && arg1.indicator) {
            // Called from Dispatcher: update_indicator(chart, cmd)
            indicatorName = arg1.indicator;
            newParams = arg1.params;
        }

        if (indicatorName && newParams) {
            const meta = window.indicatorsMetadata.get(indicatorName);
            if (meta) {
                // Update local params
                meta.params = newParams;
                
                // Emit to Python backend
                const invoke = window.__TAURI__ ? (window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke) : null;
                if (invoke) {
                    invoke('emit_to_backend', { 
                        action: 'update_indicator', 
                        data: { 
                            indicator: indicatorName,
                            ind_type: meta.indType,
                            owner_id: meta.ownerId,
                            params: newParams
                        } 
                    });
                }
            }
        }
    },
    set_available_indicators: (_targetChart, cmd) => {
        if (window.setAvailableIndicators) {
            window.setAvailableIndicators(cmd.data);
        }
    }
};

CommandQueue.processCommandSync = CommandQueue.processCommandSync.bind(CommandQueue);
window.CommandHandlers = CommandHandlers; 
window.handleCommand = handleCommand;
window.hideLoader = hideLoader;

// Final Initialization moved to entry.js
