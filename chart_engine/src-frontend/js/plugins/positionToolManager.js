/* Position Tool Manager */
const PositionToolManager = {
    _positions: new Map(),
    _rafId: null,
    _containers: new Map(),
    _dirtyCharts: new Set(),

    init: function (chartId, chartElement) {
        const container = document.createElement('div');
        container.style.position = 'absolute';
        container.style.top = '0'; container.style.left = '0';
        container.style.width = '100%'; container.style.height = '100%';
        container.style.pointerEvents = 'none'; container.style.zIndex = '101';

        if (chartElement.firstChild) {
            chartElement.insertBefore(container, chartElement.firstChild);
        } else {
            chartElement.appendChild(container);
        }

        this._containers.set(chartId, container);

        const chart = window.charts.get(chartId);
        if (chart) {
            chart.timeScale().subscribeVisibleLogicalRangeChange(() => this.updatePositions(chartId));
            chart.timeScale().subscribeSizeChange(() => this.updatePositions(chartId));
            chart.subscribeCrosshairMove(() => this.updatePositions(chartId));
        }

        chartElement.addEventListener('wheel', () => this.updatePositions(chartId), { passive: true });
        chartElement.addEventListener('touchmove', () => this.updatePositions(chartId), { passive: true });
        chartElement.addEventListener('touchstart', () => this.updatePositions(chartId), { passive: true });
    },

    create: function (chartId, id, data) {
        if (this._positions.has(id)) return;
        const container = this._containers.get(chartId);
        if (!container) return;

        data.quantity = data.quantity || 1;
        const el = document.createElement('div');
        el.style.position = 'absolute';
        el.style.display = data.visible === false ? 'none' : 'block';
        el.id = `pos-${id}`;
        el.classList.add('gpu-layer');

        const profitDiv = document.createElement('div');
        profitDiv.style.position = 'absolute';
        profitDiv.style.background = 'linear-gradient(180deg, rgba(76, 175, 80, 0.4) 0%, rgba(76, 175, 80, 0.1) 100%)';
        profitDiv.style.border = '1px solid rgba(76, 175, 80, 0.8)';
        profitDiv.style.boxSizing = 'border-box';
        profitDiv.style.borderRadius = '4px';
        profitDiv.innerHTML = '<span style="color:#ffffff; font-weight:bold; font-family:sans-serif; font-size:10px; padding:2px 4px; background:rgba(76, 175, 80, 1); border-radius:0 0 4px 0;">TP</span>';

        const lossDiv = document.createElement('div');
        lossDiv.style.position = 'absolute';
        lossDiv.style.background = 'linear-gradient(180deg, rgba(244, 67, 54, 0.1) 0%, rgba(244, 67, 54, 0.4) 100%)';
        lossDiv.style.border = '1px solid rgba(244, 67, 54, 0.8)';
        lossDiv.style.boxSizing = 'border-box';
        lossDiv.style.borderRadius = '4px';
        lossDiv.innerHTML = '<span style="color:#ffffff; font-weight:bold; font-family:sans-serif; font-size:10px; padding:2px 4px; background:rgba(244, 67, 54, 1); border-radius:0 0 4px 0;">SL</span>';

        const entryDiv = document.createElement('div');
        entryDiv.style.position = 'absolute';
        entryDiv.style.height = '1px';
        entryDiv.style.backgroundColor = '#B0BEC5';
        entryDiv.style.borderTop = '1px dashed #ffffff';
        entryDiv.style.opacity = '0.8';

        const statsDiv = document.createElement('div');
        statsDiv.style.position = 'absolute';
        statsDiv.style.color = '#ffffff';
        statsDiv.style.fontFamily = "'Inter', sans-serif";
        statsDiv.style.fontSize = '11px';
        statsDiv.style.background = 'rgba(30, 30, 30, 0.9)';
        statsDiv.style.padding = '8px';
        statsDiv.style.borderRadius = '6px';
        statsDiv.style.border = '1px solid rgba(255, 255, 255, 0.1)';
        statsDiv.style.whiteSpace = 'nowrap';
        statsDiv.style.pointerEvents = 'none';
        statsDiv.style.boxShadow = '0 4px 10px rgba(0,0,0,0.4)';
        statsDiv.style.display = 'flex';
        statsDiv.style.flexDirection = 'column';
        statsDiv.style.gap = '3px';
        statsDiv.style.zIndex = '20';
        statsDiv.style.minWidth = '120px';

        el.appendChild(profitDiv); el.appendChild(lossDiv); el.appendChild(entryDiv); el.appendChild(statsDiv);
        container.appendChild(el);

        const position = {
            id: id, chartId: chartId, data: data, element: el,
            profitEl: profitDiv, lossEl: lossDiv, entryEl: entryDiv, statsEl: statsDiv
        };
        this._positions.set(id, position);
        this.updatePosition(position);
    },

    remove: function (id) {
        const pos = this._positions.get(id);
        if (pos) {
            if (pos.element.parentNode) pos.element.parentNode.removeChild(pos.element);
            this._positions.delete(id);
        }
    },

    removeAll: function () {
        this._positions.forEach(pos => {
            if (pos.element.parentNode) pos.element.parentNode.removeChild(pos.element);
        });
        this._positions.clear();
    },

    update: function (id, partialData) {
        const pos = this._positions.get(id);
        if (pos) {
            Object.assign(pos.data, partialData);
            if (partialData.visible !== undefined) pos.element.style.display = partialData.visible ? 'block' : 'none';
            this.updatePosition(pos);
        }
    },

    updatePositions: function (specificChartId) {
        if (specificChartId) this._dirtyCharts.add(specificChartId);
        if (this._rafId) return;
        this._rafId = requestAnimationFrame(() => {
            this._positions.forEach(pos => {
                if (this._dirtyCharts.size === 0 || this._dirtyCharts.has(pos.chartId)) {
                    this.updatePosition(pos);
                }
            });
            this._dirtyCharts.clear(); this._rafId = null;
        });
    },

    updatePosition: function (pos) {
        const chart = window.charts.get(pos.chartId);
        if (!chart) return;
        const series = window.getSeriesForChart(pos.chartId);
        if (!series) return;

        const timeScale = chart.timeScale();
        const data = pos.data;
        const container = this._containers.get(pos.chartId);
        const containerW = container ? container.clientWidth : 2000;

        let x1 = null;
        if (data.start_time != null) {
            const c = timeScale.timeToCoordinate(data.start_time);
            if (c !== null) x1 = c;
            else {
                const range = timeScale.getVisibleRange();
                if (range && data.start_time < range.from) x1 = 0;
            }
        }
        let x2 = containerW;
        if (data.end_time != null) {
            const c = timeScale.timeToCoordinate(data.end_time);
            x2 = c !== null ? c : containerW;
        }

        const yEntry = series.priceToCoordinate(data.entry_price);
        const ySL = series.priceToCoordinate(data.sl_price);
        const yTP = series.priceToCoordinate(data.tp_price);

        if (x1 === null || yEntry === null || ySL === null || yTP === null) {
            pos.element.style.display = 'none'; return;
        }

        if (data.visible === false) { pos.element.style.display = 'none'; return; }
        pos.element.style.display = 'block';

        const left = Math.min(x1, x2);
        const width = Math.abs(x2 - x1);
        const yProfitTop = Math.min(yTP, yEntry);
        const hProfit = Math.abs(yTP - yEntry);
        const yLossTop = Math.min(ySL, yEntry);
        const hLoss = Math.abs(ySL - yEntry);

        pos.profitEl.style.left = left + 'px'; pos.profitEl.style.width = width + 'px';
        pos.profitEl.style.top = yProfitTop + 'px'; pos.profitEl.style.height = hProfit + 'px';

        pos.lossEl.style.left = left + 'px'; pos.lossEl.style.width = width + 'px';
        pos.lossEl.style.top = yLossTop + 'px'; pos.lossEl.style.height = hLoss + 'px';

        pos.entryEl.style.left = left + 'px'; pos.entryEl.style.width = width + 'px';
        pos.entryEl.style.top = (yEntry) + 'px';

        const risk = Math.abs(data.entry_price - data.sl_price);
        const reward = Math.abs(data.tp_price - data.entry_price);
        const rr = risk !== 0 ? (reward / risk).toFixed(2) : '∞';
        const qty = data.quantity || 1;
        const pnl = (data.type === 'long' ? (data.tp_price - data.entry_price) : (data.entry_price - data.tp_price)) * qty;
        const riskAmt = risk * qty;
        const typeStr = data.type === 'long' ? 'LONG' : 'SHORT';
        const typeColor = data.type === 'long' ? '#4CAF50' : '#F44336';

        pos.statsEl.innerHTML = `
            <div style="display:flex; justify-content:space-between; margin-bottom:4px; border-bottom:1px solid rgba(255,255,255,0.1); padding-bottom:2px">
                <span style="color:${typeColor}; font-weight:bold">${typeStr}</span>
                <span style="color:#aaa">Qty: ${qty}</span>
            </div>
            <div style="display:grid; grid-template-columns: auto auto; gap: 2px 10px;">
                <span style="color:#ccc">Entry:</span> <span style="text-align:right">${data.entry_price.toFixed(2)}</span>
                <span style="color:#4CAF50">TP:</span> <span style="text-align:right">${data.tp_price.toFixed(2)}</span>
                <span style="color:#EF5350">SL:</span> <span style="text-align:right">${data.sl_price.toFixed(2)}</span>
            </div>
            <div style="margin-top:4px; padding-top:2px; border-top:1px solid rgba(255,255,255,0.1); display:grid; grid-template-columns: auto auto; gap: 2px 10px;">
                <span style="color:#ccc">Ratio:</span> <span style="text-align:right; font-weight:bold">${rr}</span>
                <span style="color:#4CAF50">Reward:</span> <span style="text-align:right">$${pnl.toFixed(2)}</span>
                <span style="color:#EF5350">Risk:</span> <span style="text-align:right">$${riskAmt.toFixed(2)}</span>
            </div>
        `;

        pos.statsEl.style.left = (left - 8) + 'px';
        const centerY = (Math.min(yEntry, yTP, ySL) + Math.max(yEntry, yTP, ySL)) / 2;
        pos.statsEl.style.top = centerY + 'px';
        pos.statsEl.style.transform = 'translate(-100%, -50%)';
    }
};

window.PositionToolManager = PositionToolManager;
