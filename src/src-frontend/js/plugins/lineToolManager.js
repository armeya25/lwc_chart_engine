/* Line Tool Manager */
const LineToolManager = {
    _tools: new Map(),
    _rafId: null,
    _containers: new Map(),
    _dirtyCharts: new Set(),

    init: function (chartId, chartElement) {
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        svg.style.position = 'absolute'; svg.style.top = '0'; svg.style.left = '0';
        svg.style.width = '100%'; svg.style.height = '100%';
        svg.style.pointerEvents = 'none'; svg.style.zIndex = '102';

        if (chartElement.firstChild) {
            chartElement.insertBefore(svg, chartElement.firstChild);
        } else {
            chartElement.appendChild(svg);
        }

        this._containers.set(chartId, svg);

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
        const svg = this._containers.get(chartId);
        if (!svg) return;
        const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
        g.id = `line-${id}`;
        g.style.display = data.visible === false ? 'none' : 'block';
        svg.appendChild(g);

        const tool = { id: id, chartId: chartId, data: data, element: g };
        this._tools.set(id, tool);
        this.updateTool(tool);
    },

    remove: function (id) {
        const tool = this._tools.get(id);
        if (tool) {
            if (tool.element.parentNode) tool.element.parentNode.removeChild(tool.element);
            this._tools.delete(id);
        }
    },

    removeAll: function () {
        this._tools.forEach(tool => {
            if (tool.element.parentNode) tool.element.parentNode.removeChild(tool.element);
        });
        this._tools.clear();
    },

    update: function (id, partialData) {
        const tool = this._tools.get(id);
        if (tool) {
            Object.assign(tool.data, partialData);
            if (partialData.visible !== undefined) tool.element.style.display = partialData.visible ? 'block' : 'none';
            this.updateTool(tool);
        }
    },

    updatePositions: function (specificChartId) {
        if (specificChartId) this._dirtyCharts.add(specificChartId);
        if (this._rafId) return;
        this._rafId = requestAnimationFrame(() => {
            this._tools.forEach(tool => {
                if (this._dirtyCharts.size === 0 || this._dirtyCharts.has(tool.chartId)) {
                    this.updateTool(tool);
                }
            });
            this._dirtyCharts.clear(); this._rafId = null;
        });
    },

    updateTool: function (tool) {
        const chart = window.charts.get(tool.chartId);
        if (!chart) return;
        const series = window.getSeriesForChart(tool.chartId);
        if (!series) return;

        const timeScale = chart.timeScale();
        const data = tool.data;
        const g = tool.element;

        while (g.firstChild) { g.removeChild(g.firstChild); }

        const container = this._containers.get(tool.chartId);
        const w = container ? container.clientWidth : 2000;

        const getX = (t) => {
            const coord = timeScale.timeToCoordinate(t);
            if (coord !== null) return coord;
            const range = timeScale.getVisibleRange();
            if (!range) return null;
            if (t < range.from) return -10000;
            if (t > range.to) return w + 10000;
            return null;
        };

        const x1 = getX(data.start_time);
        const y1 = series.priceToCoordinate(data.start_price);
        const x2 = getX(data.end_time);
        const y2 = series.priceToCoordinate(data.end_price);

        if (x1 === null || y1 === null) return;

        const color = data.color || '#2196F3';
        const width = data.width || 2;
        let dash = '';
        if (data.style === 1) dash = '5,5';
        if (data.style === 2) dash = '2,2';

        if (data.type === 'trendline') {
            if (x2 !== null && y2 !== null) {
                this.drawSvgLine(g, x1, y1, x2, y2, color, width, dash, true);
                if (data.text) this.drawTextLabel(g, x1, y1, x2, y2, data.text, color);
            }
        } else if (data.type === 'ray') {
            if (x2 !== null && y2 !== null) {
                const dx = x2 - x1; const dy = y2 - y1;
                if (dx === 0 && dy === 0) return;
                const factor = 10000;
                this.drawSvgLine(g, x1, y1, x1 + dx * factor, y1 + dy * factor, color, width, dash, true);
                if (data.text) this.drawTextLabel(g, x1, y1, x2, y2, data.text, color);
            }
        } else if (data.type === 'fib') {
            if (x2 !== null && y2 !== null) {
                const levels = [0, 0.236, 0.382, 0.5, 0.618, 0.786, 1];
                const colors = ['rgba(244,67,54,0.15)', 'rgba(255,152,0,0.15)', 'rgba(255,235,59,0.15)', 'rgba(205,220,57,0.15)', 'rgba(76,175,80,0.15)', 'rgba(0,150,136,0.15)'];

                let leftX = Math.min(x1, x2);
                let rightX = data.extended ? (container ? container.clientWidth : 2000) : Math.max(x1, x2);

                for (let i = 0; i < levels.length - 1; i++) {
                    const yT = y1 + (y2 - y1) * levels[i];
                    const yB = y1 + (y2 - y1) * levels[i + 1];
                    const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
                    rect.setAttribute("x", leftX); rect.setAttribute("y", Math.min(yT, yB));
                    rect.setAttribute("width", Math.abs(rightX - leftX)); rect.setAttribute("height", Math.abs(yB - yT));
                    rect.setAttribute("fill", colors[i] || 'rgba(255,255,255,0.05)'); rect.setAttribute("stroke", "none");
                    g.appendChild(rect);
                }

                levels.forEach(level => {
                    const levelY = y1 + (y2 - y1) * level;
                    const lineColor = (level === 0 || level === 1) ? color : 'rgba(255,255,255,0.4)';
                    this.drawSvgLine(g, leftX, levelY, rightX, levelY, lineColor, 1, dash);
                    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
                    text.setAttribute("x", leftX + 4); text.setAttribute("y", levelY - 4);
                    text.setAttribute("fill", lineColor); text.setAttribute("font-size", "10");
                    text.setAttribute("font-family", "sans-serif");
                    text.textContent = `${level} (${(data.start_price + (data.end_price - data.start_price) * level).toFixed(2)})`;
                    g.appendChild(text);
                });
            }
        }
    },

    drawSvgLine: function (g, x1, y1, x2, y2, color, width, dash, glow = false) {
        const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
        line.setAttribute("x1", x1); line.setAttribute("y1", y1);
        line.setAttribute("x2", x2); line.setAttribute("y2", y2);
        line.setAttribute("stroke", color); line.setAttribute("stroke-width", width);
        if (dash) line.setAttribute("stroke-dasharray", dash);
        if (glow) line.style.filter = `drop-shadow(0 0 3px ${color})`;
        g.appendChild(line);
    },

    drawTextLabel: function (g, x1, y1, x2, y2, text, color) {
        const midX = (x1 + x2) / 2; const midY = (y1 + y2) / 2;
        const angle = Math.atan2(y2 - y1, x2 - x1) * (180 / Math.PI);
        let rotation = angle > 90 || angle < -90 ? angle + 180 : angle;
        const t = document.createElementNS("http://www.w3.org/2000/svg", "text");
        t.setAttribute("x", midX); t.setAttribute("y", midY - 6);
        t.setAttribute("fill", color); t.setAttribute("font-size", "11");
        t.setAttribute("font-family", "sans-serif"); t.setAttribute("font-weight", "bold");
        t.setAttribute("text-anchor", "middle"); t.setAttribute("transform", `rotate(${rotation}, ${midX}, ${midY})`);
        t.style.textShadow = '0 0 2px rgba(0,0,0,0.8)'; t.textContent = text;
        g.appendChild(t);
    }
};

window.LineToolManager = LineToolManager;
