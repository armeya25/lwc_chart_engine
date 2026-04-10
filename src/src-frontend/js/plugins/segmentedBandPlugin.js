class SegmentedBandRenderer {
    constructor(data, options) {
        this.data = data;
        this.options = options;
    }
    draw(target) {
        if (!this.data || this.data.length === 0) return;
        target.useBitmapCoordinateSpace((scope) => {
            const ctx = scope.context;
            ctx.scale(scope.horizontalPixelRatio, scope.verticalPixelRatio);
            
            const points = this.data;
            if (points.length < 2) return;

            const opacity = this.options.opacity !== undefined ? this.options.opacity : 0.2;
            
            for (let i = 1; i < points.length; i++) {
                const prev = points[i - 1];
                const curr = points[i];
                
                if (prev.y1 === null || prev.y2 === null || curr.y1 === null || curr.y2 === null) continue;
                
                // Gap detection: if points are too far apart in px, don't connect
                const dx = Math.abs(curr.x - prev.x);
                if (dx > 200) continue; 

                ctx.beginPath();
                ctx.moveTo(prev.x, prev.y1);
                ctx.lineTo(curr.x, curr.y1);
                ctx.lineTo(curr.x, curr.y2);
                ctx.lineTo(prev.x, prev.y2);
                ctx.closePath();
                
                // Apply color and opacity
                let baseColor = curr.color || this.options.color || '#2196F3';
                ctx.fillStyle = this.hexToRgba(baseColor, opacity);
                ctx.fill();
            }
        });
    }

    hexToRgba(hex, alpha) {
        if (hex.startsWith('rgba')) return hex; // Already rgba
        if (hex.startsWith('rgb')) return hex.replace(')', `, ${alpha})`).replace('rgb', 'rgba');
        
        let r = 0, g = 0, b = 0;
        if (hex.length === 4) {
            r = parseInt(hex[1] + hex[1], 16);
            g = parseInt(hex[2] + hex[2], 16);
            b = parseInt(hex[3] + hex[3], 16);
        } else if (hex.length === 7) {
            r = parseInt(hex.substring(1, 3), 16);
            g = parseInt(hex.substring(3, 5), 16);
            b = parseInt(hex.substring(5, 7), 16);
        }
        return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }

    drawBackground() {}
}

class SegmentedBandPaneView {
    constructor(source) {
        this.source = source;
        this._views = [];
    }
    update() {
        if (!this.source.chart || !this.source.series) return;
        const timeScale = this.source.chart.timeScale();
        const series = this.source.series;
        this._views = [];

        for (const pt of this.source.data) {
            const x = timeScale.timeToCoordinate(pt.time);
            if (x === null) continue;

            const y1 = series.priceToCoordinate(pt.top);
            const y2 = series.priceToCoordinate(pt.bottom);
            
            if (y1 !== null && y2 !== null) {
                this._views.push({ x, y1, y2, color: pt.color });
            }
        }
    }
    renderer() {
        return new SegmentedBandRenderer(this._views, this.source.options);
    }
}

export class SegmentedBandPrimitive {
    constructor(options = {}) {
        this.options = options;
        this.data = [];
        this._paneViews = [new SegmentedBandPaneView(this)];
    }
    attached({ chart, series, requestUpdate }) {
        this.chart = chart;
        this.series = series;
        this.requestUpdate = requestUpdate;
    }
    detached() {
        this.chart = null;
        this.series = null;
    }
    updateAllViews() {
        this._paneViews.forEach(v => v.update());
    }
    paneViews() { return this._paneViews; }
    setData(data) {
        this.data = data;
        if (this.requestUpdate) this.requestUpdate();
    }
}
