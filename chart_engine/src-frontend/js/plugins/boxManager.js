/* Box/Rectangle Overlay Manager */
const BoxManager = {
    _boxes: new Map(),
    _rafId: null,
    _containers: new Map(), // chartId -> container
    _dirtyCharts: new Set(),

    init: function (chartId, chartElement) {
        // Create an overlay layer on top of the chart cell
        const container = document.createElement('div');
        container.style.position = 'absolute';
        container.style.top = '0';
        container.style.left = '0';
        container.style.width = '100%';
        container.style.height = '100%';
        container.style.pointerEvents = 'none'; // click-through
        container.style.zIndex = '9999';

        chartElement.appendChild(container);

        this._containers.set(chartId, container);

        // Subscribe to chart updates to redraw
        const chart = window.charts.get(chartId);
        if (chart) {
            chart.timeScale().subscribeVisibleLogicalRangeChange(() => this.updatePositions(chartId));
            chart.timeScale().subscribeSizeChange(() => this.updatePositions(chartId));
            // Hook into crosshair move to catch Price Scale interactions (Y-axis zoom/scroll)
            chart.subscribeCrosshairMove(() => this.updatePositions(chartId));
        }

        // Add native DOM listeners for pinch/zoom interactions that might bypass LWC events
        chartElement.addEventListener('wheel', () => this.updatePositions(chartId), { passive: true });
        chartElement.addEventListener('touchmove', () => this.updatePositions(chartId), { passive: true });
        chartElement.addEventListener('touchstart', () => this.updatePositions(chartId), { passive: true });
    },

    createBox: function (chartId, id, data) {
        const container = this._containers.get(chartId);
        if (!container) {
            return;
        }

        const div = document.createElement('div');
        div.style.position = 'absolute';
        div.style.backgroundColor = data.color;

        const width = data.border_width !== undefined ? data.border_width : 1;
        const style = data.border_style || 'solid';
        div.style.border = `${width}px ${style} ${data.border_color || data.color}`;
        div.style.opacity = '0.8';
        div.style.boxSizing = 'border-box';
        div.style.display = data.visible === false ? 'none' : 'block';
        div.id = `box-${id}`;
        div.classList.add('gpu-layer');

        if (data.text) {
            div.innerText = data.text;
            div.style.color = data.text_color || '#ffffff';
            div.style.display = 'flex';
            div.style.alignItems = 'center';
            div.style.justifyContent = 'center';
            div.style.fontSize = '12px';
            div.style.overflow = 'hidden';
            div.style.whiteSpace = 'nowrap';
        }

        container.appendChild(div);

        const box = { id: id, chartId: chartId, data: data, element: div };
        this._boxes.set(id, box);
        this.updateBoxPosition(box);
    },

    removeBox: function (id) {
        const box = this._boxes.get(id);
        if (box) {
            if (box.element.parentNode) box.element.parentNode.removeChild(box.element);
            this._boxes.delete(id);
        }
    },

    removeAll: function () {
        this._boxes.forEach(box => {
            if (box.element.parentNode) box.element.parentNode.removeChild(box.element);
        });
        this._boxes.clear();
    },

    updateBox: function (id, partialData) {
        const box = this._boxes.get(id);
        if (box) {
            Object.assign(box.data, partialData);
            if (partialData.color) box.element.style.backgroundColor = partialData.color;
            if (partialData.border_color || partialData.border_width !== undefined || partialData.border_style) {
                const width = box.data.border_width !== undefined ? box.data.border_width : 1;
                const style = box.data.border_style || 'solid';
                const color = box.data.border_color || box.data.color;
                box.element.style.border = `${width}px ${style} ${color}`;
            }
            if (partialData.visible !== undefined) box.element.style.display = partialData.visible ? 'block' : 'none';
            if (partialData.text !== undefined) {
                box.element.innerText = partialData.text;
                if (partialData.text) {
                    box.element.style.display = (box.data.visible !== false) ? 'flex' : 'none';
                    box.element.style.alignItems = 'center';
                    box.element.style.justifyContent = 'center';
                }
            }
            if (partialData.text_color) box.element.style.color = partialData.text_color;

            this.updateBoxPosition(box);
        }
    },

    updatePositions: function (specificChartId) {
        if (specificChartId) this._dirtyCharts.add(specificChartId);

        if (this._rafId) return;
        this._rafId = requestAnimationFrame(() => {
            this._boxes.forEach(box => {
                if (this._dirtyCharts.size === 0 || this._dirtyCharts.has(box.chartId)) {
                    this.updateBoxPosition(box);
                }
            });
            this._dirtyCharts.clear();
            this._rafId = null;
        });
    },

    updateBoxPosition: function (box) {
        const chart = window.charts.get(box.chartId);
        if (!chart) return;
        const series = window.getSeriesForChart(box.chartId);
        if (!series) {
            // window.bridgeLog(`BoxManager ERROR: No series for [${box.chartId}]`);
            return;
        }

        const timeScale = chart.timeScale();
        const container = this._containers.get(box.chartId);
        const data = box.data;

        const getX = (t) => {
            if (t == null) return null;
            let coord = timeScale.timeToCoordinate(t);
            if (coord !== null) return coord;
            
            // FUZZY MATCH: If exact time fails, check if it's out of range
            const range = timeScale.getVisibleRange();
            if (!range) {
                 // Chart might be early in loading
                 return null;
            }
            if (t < range.from) return -10000;
            if (t > range.to) return (container ? container.clientWidth : 1000) + 10000;
            
            // If it's in range but null, it's between bars. 
            // Try to approximate by finding the coordinate of the closest bar.
            return null;
        };

        const x1 = getX(data.start_time);
        const x2 = getX(data.end_time);
        const p1 = data.top_price !== undefined ? data.top_price : data.start_price;
        const p2 = data.bottom_price !== undefined ? data.bottom_price : data.end_price;
        const y1 = series.priceToCoordinate(p1);
        const y2 = series.priceToCoordinate(p2);

        if (x1 === null || y1 === null) {
            // window.bridgeLog(`BoxManager: Box [${box.id}] hidden - X1=${x1} Y1=${y1}`);
            box.element.style.display = 'none'; 
            return;
        }

        box.element.style.display = (data.visible !== false) ? 'block' : 'none';

        const resolvedX2 = x2 !== null ? x2 : (container ? container.clientWidth + 9999 : 9999);
        const resolvedY2 = y2 !== null ? y2 : y1;

        const left = Math.min(x1, resolvedX2);
        const width = Math.abs(resolvedX2 - x1);
        const top = Math.min(y1, resolvedY2);
        const height = Math.abs(resolvedY2 - y1);

        box.element.style.left = `${left}px`;
        box.element.style.top = `${top}px`;
        box.element.style.width = `${width}px`;
        box.element.style.height = `${height}px`;
    }
};

window.BoxManager = BoxManager;
