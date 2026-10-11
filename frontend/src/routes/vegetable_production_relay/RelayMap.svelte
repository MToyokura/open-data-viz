<script lang="ts">
  import { onMount, untrack } from "svelte";

  import type { PrefectureMonth } from "./constants/types";
  // here
  type PrefectureObservation = PrefectureMonth;
  type GeoCollection = {
    features: Array<{ properties?: { N03_001?: string }; geometry: unknown }>;
  };

  let {
    geojson,
    observations,
    sourceUnit,
    scale,
    onSelect,
  }: {
    geojson: GeoCollection;
    observations: Record<string, PrefectureObservation>;
    sourceUnit: string | null;
    scale: number;
    onSelect: (name: string) => void;
  } = $props();

  let host: HTMLDivElement;
  let mapReady = $state(false);
  let leaflet: typeof import("leaflet") | undefined;
  let map: import("leaflet").Map | undefined;
  let boundary: import("leaflet").GeoJSON | undefined;
  let circles: import("leaflet").LayerGroup | undefined;
  const markers = new Map<string, import("leaflet").CircleMarker>();
  const markerAnimations = new Map<string, number>();
  const markerTargets = new Map<string, { radius: number; opacity: number }>();
  // Geographic data is recomputed only when the GeoJSON object changes.
  const centroidLocations = new Map<string, [number, number]>();
  let geometryVersion = $state(0);

  function animateMarker(
    name: string,
    marker: import("leaflet").CircleMarker,
    targetRadius: number,
    targetOpacity: number,
  ) {
    const previousTarget = markerTargets.get(name);
    const hasActiveAnimation = markerAnimations.has(name);
    if (
      previousTarget?.radius === targetRadius &&
      previousTarget.opacity === targetOpacity &&
      (hasActiveAnimation || Math.abs(marker.getRadius() - targetRadius) < 0.01)
    ) {
      return;
    }
    markerTargets.set(name, { radius: targetRadius, opacity: targetOpacity });

    const previousFrame = markerAnimations.get(name);
    if (previousFrame !== undefined) cancelAnimationFrame(previousFrame);

    const startRadius = marker.getRadius();
    const startOpacity = Number(marker.options.fillOpacity ?? 0.85);
    const startTime = performance.now();
    const duration = 250;

    const step = (now: number) => {
      const progress = Math.min((now - startTime) / duration, 1);
      const eased =
        progress < 0.5 ? 4 * progress ** 3 : 1 - (-2 * progress + 2) ** 3 / 2;
      marker.setRadius(startRadius + (targetRadius - startRadius) * eased);
      marker.setStyle({
        fillOpacity: startOpacity + (targetOpacity - startOpacity) * eased,
      });

      if (progress < 1) {
        markerAnimations.set(name, requestAnimationFrame(step));
      } else {
        markerAnimations.delete(name);
      }
    };

    markerAnimations.set(name, requestAnimationFrame(step));
  }

  function prefectureKey(name: string): string {
    return name === "北海道" ? name : name.replace(/[都府県]$/, "");
  }

  function formatQuantity(value: string, unit: string | null): string {
    return `${value} ${unit ?? "単位未確認"}`;
  }

  function observationLabel(observation: PrefectureObservation | undefined) {
    if (!observation) return "値なし";
    if (observation.reported_cells > 0) {
      const quantity = formatQuantity(
        observation.reported_value_sum,
        sourceUnit,
      );
      return observation.status === "incomplete"
        ? `${quantity}（一部未集計）`
        : quantity;
    }
    if (observation.below_display_unit_cells > 0) return "表示単位未満";
    if (observation.unknown_or_uncollected_cells > 0) return "不詳・調査なし";
    if (observation.no_fact_cells > 0) return "該当なし";
    return "値なし";
  }

  function ringCentroid(
    ring: unknown,
  ): { x: number; y: number; area: number } | null {
    if (!Array.isArray(ring) || ring.length < 4) return null;
    let twiceArea = 0;
    let x = 0;
    let y = 0;
    for (let index = 0; index < ring.length - 1; index += 1) {
      const first = ring[index];
      const second = ring[index + 1];
      if (!Array.isArray(first) || !Array.isArray(second)) continue;
      const cross =
        Number(first[0]) * Number(second[1]) -
        Number(second[0]) * Number(first[1]);
      twiceArea += cross;
      x += (Number(first[0]) + Number(second[0])) * cross;
      y += (Number(first[1]) + Number(second[1])) * cross;
    }
    if (!Number.isFinite(twiceArea) || Math.abs(twiceArea) < 1e-12) return null;
    return {
      x: x / (3 * twiceArea),
      y: y / (3 * twiceArea),
      area: Math.abs(twiceArea / 2),
    };
  }

  function polygonCentroid(
    polygon: unknown,
  ): { x: number; y: number; area: number } | null {
    if (!Array.isArray(polygon)) return null;
    let weight = 0;
    let x = 0;
    let y = 0;
    polygon.forEach((ring, index) => {
      const centroid = ringCentroid(ring);
      if (!centroid) return;
      const area = index === 0 ? centroid.area : -centroid.area;
      weight += area;
      x += centroid.x * area;
      y += centroid.y * area;
    });
    return weight > 0 ? { x: x / weight, y: y / weight, area: weight } : null;
  }

  onMount(() => {
    let cancelled = false;
    void (async () => {
      const L = await import("leaflet");
      await import("leaflet/dist/leaflet.css");
      if (cancelled || !host.isConnected) return;
      leaflet = L;
      map = L.map(host, {
        zoomControl: true,
        attributionControl: true,
        zoomSnap: 0.5,
        scrollWheelZoom: true,
        wheelPxPerZoomLevel: 240,
      });
      map.attributionControl.setPrefix("Leaflet");
      map.attributionControl.addAttribution(
        '<a href="https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2026.html" target="_blank" rel="noreferrer">国土交通省「国土数値情報 行政区域データ（N03）」2026年版</a>',
      );
      map.setView([37, 137], 5);
      mapReady = true;
    })();
    return () => {
      cancelled = true;
      mapReady = false;
      for (const frame of markerAnimations.values()) {
        cancelAnimationFrame(frame);
      }
      markerAnimations.clear();
      markers.clear();
      markerTargets.clear();
      map?.remove();
      map = undefined;
    };
  });

  // Static geometry: rebuild boundaries and centroid coordinates only when
  // the map is initialized or a different GeoJSON object is supplied.
  $effect(() => {
    if (!mapReady || !leaflet || !map || !geojson) return;

    boundary?.remove();
    boundary = leaflet
      .geoJSON(geojson as never, {
        style: () => ({
          color: "#4b5563",
          weight: 1,
          fillColor: "#f1f3f5",
          fillOpacity: 1,
        }),
        onEachFeature: (feature, layer) => {
          const name = prefectureKey(
            String(feature.properties?.N03_001 ?? "").trim(),
          );
          // The observation effect supplies the actual label.
          layer.bindTooltip(`${name}: 値なし`);
          layer.on("click", () => onSelect(name));
        },
      })
      .addTo(map);

    circles ??= leaflet.layerGroup().addTo(map);
    const centroids = new Map<string, { area: number; x: number; y: number }>();
    for (const feature of geojson.features) {
      const name = prefectureKey(
        String(feature.properties?.N03_001 ?? "").trim(),
      );
      if (!name) continue;
      const geometry = feature.geometry as {
        type: string;
        coordinates: unknown;
      };
      const polygons: unknown[] =
        geometry.type === "Polygon"
          ? [geometry.coordinates]
          : geometry.type === "MultiPolygon" &&
              Array.isArray(geometry.coordinates)
            ? geometry.coordinates
            : [];
      const aggregate = centroids.get(name) ?? { area: 0, x: 0, y: 0 };
      for (const polygon of polygons) {
        const centroid = polygonCentroid(polygon);
        if (!centroid) continue;
        aggregate.area += centroid.area;
        aggregate.x += centroid.x * centroid.area;
        aggregate.y += centroid.y * centroid.area;
      }
      centroids.set(name, aggregate);
    }

    centroidLocations.clear();
    for (const [name, centroid] of centroids) {
      if (centroid.area > 0) {
        centroidLocations.set(name, [
          centroid.y / centroid.area,
          centroid.x / centroid.area,
        ]);
      }
    }

    // Existing markers must stay above the replacement boundary layer.
    for (const marker of markers.values()) marker.bringToFront();

    const bounds = boundary.getBounds();
    if (bounds.isValid()) map.fitBounds(bounds, { padding: [12, 12] });

    // Signal that the marker effect should consume the new locations.
    untrack(() => {
      geometryVersion += 1;
    });
  });

  // Dynamic data: refresh labels, positions, and marker animations without
  // touching boundary polygons or recalculating geographic centroids.
  $effect(() => {
    if (!mapReady || !leaflet || !map || !circles || !geometryVersion) return;
    const currentObservations = observations;
    const currentScale = scale;

    boundary?.eachLayer((layer) => {
      const feature = (
        layer as import("leaflet").Layer & {
          feature?: { properties?: { N03_001?: string } };
        }
      ).feature;
      const name = prefectureKey(
        String(feature?.properties?.N03_001 ?? "").trim(),
      );
      const observation = currentObservations[name];
      const label = `${name}: ${observationLabel(observation)}`;
      layer.getTooltip()?.setContent(label);
    });

    const activeNames = new Set<string>();
    for (const [name, location] of centroidLocations) {
      const observation = currentObservations[name];
      const value = Number(observation?.reported_value_sum ?? 0);
      if (!observation?.reported_cells || value <= 0) continue;
      activeNames.add(name);
      const targetRadius = (Math.sqrt(value) * currentScale) / 15;
      let marker = markers.get(name);

      if (marker) {
        marker.setLatLng(location);
        marker.setTooltipContent(`${name}: ${observationLabel(observation)}`);
      } else {
        marker = leaflet.circleMarker(location, {
          radius: 0,
          color: "#7f2704",
          weight: 1,
          fillColor: "#d55e00",
          fillOpacity: 0,
        });
        marker
          .bindTooltip(`${name}: ${observationLabel(observation)}`)
          .on("click", () => onSelect(name))
          .addTo(circles);
        markers.set(name, marker);
      }

      animateMarker(name, marker, targetRadius, 0.85);
    }

    for (const [name, marker] of markers) {
      if (activeNames.has(name)) continue;
      const frame = markerAnimations.get(name);
      if (frame !== undefined) cancelAnimationFrame(frame);
      markerAnimations.delete(name);
      markerTargets.delete(name);
      marker.remove();
      markers.delete(name);
    }
  });
</script>

<div
  class="map"
  bind:this={host}
  aria-label="都道府県別の野菜卸売数量地図"
></div>

<style>
  .map {
    height: min(68vh, 680px);
    min-height: 400px;
    border: 1px solid #ddd;
    border-radius: 6px;
    background: #f5f5f2;
  }
  @media (max-width: 600px) {
    .map {
      height: 55vh;
      min-height: 360px;
    }
  }
</style>
