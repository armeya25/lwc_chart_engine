class SegmentedLineRenderer {
    constructor(data, options) {
        this.data = data;
        this.options = options;
    }
    draw(target) {
        if (!this.data || this.data.length === 0) return;
        target.useBitmapCoordinateSpace((scope) => {
            const ctx = scope.context;
            ctx.scale(scope.horizontalPixelRatio, scope.verticalPixelRatio);
            ctx.lineWidth = this.options.width || 2;
            
            const points = this.data;
            if (points.length < 2) {
                console.log("SegmentedLine: Not enough points", points.length);
                return;
            }

            // console.log("SegmentedLine: Drawing points", points.length);
            
            for (let i = 1; i < points.length; i++) {
                const prev = points[i - 1];
                const curr = points[i];
                
                if (prev.y === null || curr.y === null) continue;
                
                // Gap detection: if points are too far apart in px, don't connect
                // Using a larger threshold for zoomed out views
                const dx = Math.abs(curr.x - prev.x);
                if (dx > 200) continue; 

                ctx.beginPath();
                ctx.moveTo(prev.x, prev.y);
                ctx.lineTo(curr.x, curr.y);
                
                // Use the color of the current point for the segment connecting to it
                ctx.strokeStyle = curr.color || this.options.color || '#2196F3';
                ctx.stroke();
            }
        });
    }
    drawBackground() {}
}

class SegmentedLinePaneView {
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

            const y = (pt.value !== undefined && pt.value !== null) ? series.priceToCoordinate(pt.value) : null;
            if (y !== null) {
                this._views.push({ x, y, color: pt.color });
            }
        }
    }
    renderer() {
        return new SegmentedLineRenderer(this._views, this.source.options);
    }
}

export class SegmentedLinePrimitive {
    constructor(options = {}) {
        this.options = options;
        this.data = [];
        this._paneViews = [new SegmentedLinePaneView(this)];
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
