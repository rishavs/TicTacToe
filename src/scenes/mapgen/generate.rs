use super::biome::{biome_color, calculate_lighting, get_biome, interpolate_color};
use super::*;
use std::collections::{HashMap, VecDeque};

const SHALLOW_BAY_ROUNDING_PASSES: usize = 8;

#[cfg(test)]
pub(super) fn generate_map(
    seed_text: &str,
    island_type: IslandType,
    point_type: PointType,
    point_count: usize,
) -> PolyMap {
    generate_map_with_shallow_sea(
        seed_text,
        island_type,
        point_type,
        point_count,
        DEFAULT_SHALLOW_SEA_SIZE,
    )
}

pub(super) fn generate_map_with_shallow_sea(
    seed_text: &str,
    island_type: IslandType,
    point_type: PointType,
    point_count: usize,
    shallow_sea_size: ShallowSeaSize,
) -> PolyMap {
    PolyMap::generate(
        seed_text,
        island_type,
        point_type,
        point_count,
        shallow_sea_size,
    )
}
impl PolyMap {
    fn generate(
        seed_text: &str,
        island_type: IslandType,
        point_type: PointType,
        point_count: usize,
        shallow_sea_size: ShallowSeaSize,
    ) -> Self {
        let (shape_seed, variant) = parse_seed(seed_text);
        let island_shape = IslandProfile::new(island_type, shape_seed, point_count);
        let points = select_points(point_type, point_count, shape_seed);
        let mut map = Self::from_points(points, point_count, variant, island_shape);
        map.assign_corner_elevations();
        map.assign_ocean_coast_and_land();
        map.assign_ocean_depths(shallow_sea_size);
        map.redistribute_elevations();
        map.assign_polygon_elevations();
        map.calculate_downslopes();
        map.calculate_watersheds();
        map.create_rivers();
        map.assign_corner_moisture();
        map.redistribute_moisture();
        map.assign_polygon_moisture();
        map.assign_biomes();
        map.create_center_watersheds();
        map.build_noisy_edges();
        map
    }

    fn from_points(
        points: Vec<Vec2>,
        point_count: usize,
        variant: u32,
        island_shape: IslandProfile,
    ) -> Self {
        let regions = build_regions(point_count);
        let mut centers: Vec<Center> = points
            .iter()
            .enumerate()
            .map(|(index, &point)| Center {
                index,
                point,
                water: false,
                ocean: false,
                shallow_ocean: false,
                ocean_distance: -1,
                coast: false,
                border: false,
                biome: "OCEAN",
                elevation: 0.0,
                moisture: 0.0,
                neighbors: Vec::new(),
                borders: Vec::new(),
                corners: Vec::new(),
            })
            .collect();
        let mut corners: Vec<Corner> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new();
        let mut corner_lookup: HashMap<(i32, i32), usize> = HashMap::new();
        let mut edge_by_corners: HashMap<(usize, usize), usize> = HashMap::new();

        for (center_id, region) in regions.iter().enumerate() {
            if region.len() < 3 {
                continue;
            }
            let mut corner_ids = Vec::with_capacity(region.len());
            for &point in region {
                let corner_id = make_corner(&mut corners, &mut corner_lookup, point);
                push_unique(&mut corners[corner_id].touches, center_id);
                corner_ids.push(corner_id);
            }
            centers[center_id].corners = corner_ids.clone();

            for i in 0..corner_ids.len() {
                let v0 = corner_ids[i];
                let v1 = corner_ids[(i + 1) % corner_ids.len()];
                if v0 == v1 {
                    continue;
                }
                let key = sorted_pair(v0, v1);
                let edge_id = if let Some(&edge_id) = edge_by_corners.get(&key) {
                    if edges[edge_id].d0 != Some(center_id) && edges[edge_id].d1.is_none() {
                        edges[edge_id].d1 = Some(center_id);
                    }
                    edge_id
                } else {
                    let edge_id = edges.len();
                    edge_by_corners.insert(key, edge_id);
                    edges.push(Edge {
                        index: edge_id,
                        d0: Some(center_id),
                        d1: None,
                        v0: Some(v0),
                        v1: Some(v1),
                        midpoint: (corners[v0].point + corners[v1].point) * 0.5,
                        river: 0,
                    });
                    push_unique(&mut corners[v0].adjacent, v1);
                    push_unique(&mut corners[v1].adjacent, v0);
                    push_unique(&mut corners[v0].protrudes, edge_id);
                    push_unique(&mut corners[v1].protrudes, edge_id);
                    edge_id
                };
                push_unique(&mut centers[center_id].borders, edge_id);
            }
        }

        for edge in &edges {
            if let (Some(d0), Some(d1)) = (edge.d0, edge.d1) {
                push_unique(&mut centers[d0].neighbors, d1);
                push_unique(&mut centers[d1].neighbors, d0);
            }
        }

        improve_corners(&mut centers, &mut corners, &mut edges);

        Self {
            map_random: map_rng(variant as u64),
            island_shape,
            centers,
            corners,
            edges,
            noisy_edges: Vec::new(),
            center_watersheds: Vec::new(),
            edge_by_corners,
        }
    }

    fn assign_corner_elevations(&mut self) {
        let mut queue = VecDeque::new();
        for corner in &mut self.corners {
            corner.water = !self.island_shape.inside(vec2(
                2.0 * (corner.point.x / MAP_SIZE - 0.5),
                2.0 * (corner.point.y / MAP_SIZE - 0.5),
            ));
            if corner.border {
                corner.elevation = 0.0;
                queue.push_back(corner.index);
            } else {
                corner.elevation = f32::INFINITY;
            }
        }

        while let Some(q) = queue.pop_front() {
            let adjacent = self.corners[q].adjacent.clone();
            for s in adjacent {
                let mut new_elevation = 0.01 + self.corners[q].elevation;
                if !self.corners[q].water && !self.corners[s].water {
                    new_elevation += 1.0;
                    new_elevation += map_random_f32(&mut self.map_random, 0.0..1.0);
                }
                if new_elevation < self.corners[s].elevation {
                    self.corners[s].elevation = new_elevation;
                    queue.push_back(s);
                }
            }
        }
    }

    fn assign_ocean_coast_and_land(&mut self) {
        let mut queue = VecDeque::new();
        for center in &mut self.centers {
            let mut num_water = 0usize;
            for &corner_id in &center.corners {
                let corner = &mut self.corners[corner_id];
                if corner.border {
                    center.border = true;
                    center.ocean = true;
                    corner.water = true;
                    queue.push_back(center.index);
                }
                if corner.water {
                    num_water += 1;
                }
            }
            center.water = center.ocean
                || (!center.corners.is_empty()
                    && num_water as f32 >= center.corners.len() as f32 * LAKE_THRESHOLD);
        }

        while let Some(p) = queue.pop_front() {
            let neighbors = self.centers[p].neighbors.clone();
            for r in neighbors {
                if self.centers[r].water && !self.centers[r].ocean {
                    self.centers[r].ocean = true;
                    queue.push_back(r);
                }
            }
        }

        for p in 0..self.centers.len() {
            let mut num_ocean = 0;
            let mut num_land = 0;
            for &r in &self.centers[p].neighbors {
                if self.centers[r].ocean {
                    num_ocean += 1;
                }
                if !self.centers[r].water {
                    num_land += 1;
                }
            }
            self.centers[p].coast = num_ocean > 0 && num_land > 0;
        }

        for q in 0..self.corners.len() {
            let mut num_ocean = 0;
            let mut num_land = 0;
            for &p in &self.corners[q].touches {
                if self.centers[p].ocean {
                    num_ocean += 1;
                }
                if !self.centers[p].water {
                    num_land += 1;
                }
            }
            let touches = self.corners[q].touches.len();
            self.corners[q].ocean = touches > 0 && num_ocean == touches;
            self.corners[q].coast = num_ocean > 0 && num_land > 0;
            self.corners[q].water =
                self.corners[q].border || ((num_land != touches) && !self.corners[q].coast);
        }
    }

    fn assign_ocean_depths(&mut self, shallow_sea_size: ShallowSeaSize) {
        let mut queue = VecDeque::new();
        for center in &mut self.centers {
            center.shallow_ocean = false;
            center.ocean_distance = -1;
        }

        for center_id in 0..self.centers.len() {
            if !self.centers[center_id].ocean {
                continue;
            }
            let touches_land = self.centers[center_id]
                .neighbors
                .iter()
                .any(|&neighbor| !self.centers[neighbor].water);
            if touches_land {
                self.centers[center_id].ocean_distance = 0;
                queue.push_back(center_id);
            }
        }

        while let Some(center_id) = queue.pop_front() {
            let next_distance = self.centers[center_id].ocean_distance + 1;
            let neighbors = self.centers[center_id].neighbors.clone();
            for neighbor in neighbors {
                if !self.centers[neighbor].ocean || self.centers[neighbor].ocean_distance >= 0 {
                    continue;
                }
                self.centers[neighbor].ocean_distance = next_distance;
                queue.push_back(neighbor);
            }
        }

        for center in &mut self.centers {
            if center.ocean {
                let jitter = shallow_ocean_jitter(center.point);
                let guaranteed = shallow_sea_size.guaranteed_shallow_distance();
                center.shallow_ocean = center.ocean_distance <= guaranteed
                    || (center.ocean_distance == guaranteed + 1 && jitter > 0.4);
            }
        }

        self.fill_enclosed_deep_ocean_pockets();
        self.round_shallow_bays(shallow_sea_size);
        self.fill_enclosed_deep_ocean_pockets();
        self.connect_islands_with_shallow_ocean();
        self.fill_enclosed_deep_ocean_pockets();
        self.round_shallow_bays(shallow_sea_size);
        self.fill_enclosed_deep_ocean_pockets();
    }

    fn fill_enclosed_deep_ocean_pockets(&mut self) {
        let mut visited = vec![false; self.centers.len()];
        let mut enclosed = Vec::new();

        for start in 0..self.centers.len() {
            if visited[start] || !self.centers[start].ocean || self.centers[start].shallow_ocean {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::from([start]);
            let mut touches_border = false;
            let mut bordered_only_by_shallow_ocean = true;
            visited[start] = true;

            while let Some(center_id) = queue.pop_front() {
                component.push(center_id);
                touches_border |= self.centers[center_id].border;

                for &neighbor in &self.centers[center_id].neighbors {
                    if self.centers[neighbor].ocean && !self.centers[neighbor].shallow_ocean {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    } else if !self.centers[neighbor].ocean || !self.centers[neighbor].shallow_ocean
                    {
                        bordered_only_by_shallow_ocean = false;
                    }
                }
            }

            if !touches_border && bordered_only_by_shallow_ocean {
                enclosed.extend(component);
            }
        }

        for center_id in enclosed {
            self.centers[center_id].shallow_ocean = true;
            if self.centers[center_id].ocean_distance < 0 {
                self.centers[center_id].ocean_distance = 2;
            }
        }
    }

    fn round_shallow_bays(&mut self, shallow_sea_size: ShallowSeaSize) {
        let max_distance = shallow_sea_size.guaranteed_shallow_distance() + 3;

        for _ in 0..SHALLOW_BAY_ROUNDING_PASSES {
            let rounded: Vec<_> = self
                .centers
                .iter()
                .filter(|center| self.is_shallow_bay_rounding_candidate(center.index, max_distance))
                .map(|center| center.index)
                .collect();

            if rounded.is_empty() {
                break;
            }

            for center_id in rounded {
                self.centers[center_id].shallow_ocean = true;
            }
        }
    }

    fn is_shallow_bay_rounding_candidate(&self, center_id: usize, max_distance: i32) -> bool {
        let center = &self.centers[center_id];
        center.ocean
            && !center.shallow_ocean
            && !center.border
            && center.ocean_distance <= max_distance
            && self.shallow_ocean_neighbor_count(center_id) >= 2
    }

    fn shallow_ocean_neighbor_count(&self, center_id: usize) -> usize {
        self.centers[center_id]
            .neighbors
            .iter()
            .filter(|&&neighbor| {
                self.centers[neighbor].ocean && self.centers[neighbor].shallow_ocean
            })
            .count()
    }

    fn connect_islands_with_shallow_ocean(&mut self) {
        let Some(mainland_start) = self.largest_passable_land_component_start() else {
            return;
        };

        loop {
            let connected = self.passable_component_from(mainland_start);
            if self
                .centers
                .iter()
                .all(|center| center.water || connected[center.index])
            {
                break;
            }

            let Some(path) = self.shortest_ocean_path_to_unconnected_land(&connected) else {
                break;
            };

            let radius = self.shallow_bridge_corridor_radius(&path);
            self.carve_shallow_bridge_corridor(&path, radius);
        }
    }

    fn shallow_bridge_corridor_radius(&self, path: &[usize]) -> i32 {
        let target_land_size = path
            .iter()
            .rev()
            .find(|&&center_id| !self.centers[center_id].water)
            .map(|&center_id| self.land_component_size(center_id))
            .unwrap_or(1);

        match target_land_size {
            0..=8 => 1,
            9..=24 => 2,
            _ => 3,
        }
    }

    fn land_component_size(&self, start: usize) -> usize {
        if self.centers[start].water {
            return 0;
        }

        let mut visited = vec![false; self.centers.len()];
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        let mut size = 0usize;

        while let Some(center_id) = queue.pop_front() {
            size += 1;
            for &neighbor in &self.centers[center_id].neighbors {
                if !visited[neighbor] && !self.centers[neighbor].water {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }

        size
    }

    fn carve_shallow_bridge_corridor(&mut self, path: &[usize], radius: i32) {
        let mut shallow = Vec::new();

        for &center_id in path {
            self.collect_bridge_corridor_cells(center_id, radius, &mut shallow);
        }

        shallow.sort_unstable();
        shallow.dedup();

        for center_id in shallow {
            self.centers[center_id].shallow_ocean = true;
            self.centers[center_id].ocean_distance = self.centers[center_id].ocean_distance.max(2);
        }
    }

    fn collect_bridge_corridor_cells(&self, start: usize, radius: i32, out: &mut Vec<usize>) {
        let mut visited = vec![false; self.centers.len()];
        let mut queue = VecDeque::from([(start, 0)]);
        visited[start] = true;

        while let Some((center_id, distance)) = queue.pop_front() {
            let center = &self.centers[center_id];
            if center.ocean && !center.border {
                out.push(center_id);
            }

            if distance >= radius {
                continue;
            }

            for &neighbor in &center.neighbors {
                if visited[neighbor] || self.centers[neighbor].border {
                    continue;
                }

                visited[neighbor] = true;
                queue.push_back((neighbor, distance + 1));
            }
        }
    }

    fn largest_passable_land_component_start(&self) -> Option<usize> {
        let mut visited = vec![false; self.centers.len()];
        let mut best_start = None;
        let mut best_land_count = 0usize;

        for start in 0..self.centers.len() {
            if visited[start] || !self.is_land_or_shallow_ocean(start) {
                continue;
            }

            let mut first_land = None;
            let mut land_count = 0usize;
            let mut queue = VecDeque::from([start]);
            visited[start] = true;

            while let Some(center_id) = queue.pop_front() {
                if !self.centers[center_id].water {
                    land_count += 1;
                    first_land.get_or_insert(center_id);
                }

                for &neighbor in &self.centers[center_id].neighbors {
                    if !visited[neighbor] && self.is_land_or_shallow_ocean(neighbor) {
                        visited[neighbor] = true;
                        queue.push_back(neighbor);
                    }
                }
            }

            if land_count > best_land_count {
                best_land_count = land_count;
                best_start = first_land;
            }
        }

        best_start
    }

    fn passable_component_from(&self, start: usize) -> Vec<bool> {
        let mut connected = vec![false; self.centers.len()];
        if !self.is_land_or_shallow_ocean(start) {
            return connected;
        }

        let mut queue = VecDeque::from([start]);
        connected[start] = true;
        while let Some(center_id) = queue.pop_front() {
            for &neighbor in &self.centers[center_id].neighbors {
                if !connected[neighbor] && self.is_land_or_shallow_ocean(neighbor) {
                    connected[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        connected
    }

    fn shortest_ocean_path_to_unconnected_land(&self, connected: &[bool]) -> Option<Vec<usize>> {
        let mut visited = connected.to_vec();
        let mut parent = vec![None; self.centers.len()];
        let mut queue = VecDeque::new();

        for (center_id, &is_connected) in connected.iter().enumerate() {
            if is_connected {
                queue.push_back(center_id);
            }
        }

        while let Some(center_id) = queue.pop_front() {
            for &neighbor in &self.centers[center_id].neighbors {
                if visited[neighbor] {
                    continue;
                }

                if !self.centers[neighbor].water {
                    parent[neighbor] = Some(center_id);
                    return Some(self.reconstruct_bridge_path(neighbor, &parent, connected));
                }

                if self.centers[neighbor].ocean && !self.centers[neighbor].border {
                    visited[neighbor] = true;
                    parent[neighbor] = Some(center_id);
                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    fn reconstruct_bridge_path(
        &self,
        target: usize,
        parent: &[Option<usize>],
        connected: &[bool],
    ) -> Vec<usize> {
        let mut path = Vec::new();
        let mut current = target;

        while !connected[current] {
            path.push(current);
            let Some(previous) = parent[current] else {
                break;
            };
            current = previous;
        }

        path.reverse();
        path
    }

    fn is_land_or_shallow_ocean(&self, center_id: usize) -> bool {
        let center = &self.centers[center_id];
        !center.water || (center.ocean && center.shallow_ocean)
    }

    fn redistribute_elevations(&mut self) {
        let mut locations: Vec<usize> = self
            .corners
            .iter()
            .filter(|corner| !corner.ocean && !corner.coast)
            .map(|corner| corner.index)
            .collect();
        locations.sort_by(|&a, &b| {
            self.corners[a]
                .elevation
                .total_cmp(&self.corners[b].elevation)
        });
        if locations.len() > 1 {
            let scale_factor = 1.1_f32;
            for (i, &corner_id) in locations.iter().enumerate() {
                let y = i as f32 / (locations.len() - 1) as f32;
                let x = scale_factor.sqrt() - (scale_factor * (1.0 - y)).sqrt();
                self.corners[corner_id].elevation = x.min(1.0);
            }
        }
        for corner in &mut self.corners {
            if corner.ocean || corner.coast {
                corner.elevation = 0.0;
            }
        }
    }

    fn assign_polygon_elevations(&mut self) {
        for center in &mut self.centers {
            center.elevation = average(
                center
                    .corners
                    .iter()
                    .map(|&corner_id| self.corners[corner_id].elevation),
            );
        }
    }

    fn calculate_downslopes(&mut self) {
        for q in 0..self.corners.len() {
            let mut r = q;
            for &s in &self.corners[q].adjacent {
                if self.corners[s].elevation <= self.corners[r].elevation {
                    r = s;
                }
            }
            self.corners[q].downslope = r;
        }
    }

    fn calculate_watersheds(&mut self) {
        for q in 0..self.corners.len() {
            self.corners[q].watershed = q;
            if !self.corners[q].ocean && !self.corners[q].coast {
                self.corners[q].watershed = self.corners[q].downslope;
            }
            self.corners[q].watershed_size = 0;
        }

        for _ in 0..100 {
            let mut changed = false;
            for q in 0..self.corners.len() {
                let watershed = self.corners[q].watershed;
                if !self.corners[q].ocean
                    && !self.corners[q].coast
                    && !self.corners[watershed].coast
                {
                    let r = self.corners[self.corners[q].downslope].watershed;
                    if !self.corners[r].ocean {
                        self.corners[q].watershed = r;
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        for q in 0..self.corners.len() {
            let watershed = self.corners[q].watershed;
            self.corners[watershed].watershed_size += 1;
        }
    }

    fn create_rivers(&mut self) {
        for _ in 0..(MAP_SIZE as usize / 2) {
            if self.corners.is_empty() {
                return;
            }
            let mut q =
                map_random_i32(&mut self.map_random, 0..=(self.corners.len() as i32 - 1)) as usize;
            if self.corners[q].ocean
                || self.corners[q].elevation < 0.3
                || self.corners[q].elevation > 0.9
            {
                continue;
            }
            let mut visited = vec![false; self.corners.len()];
            while !self.corners[q].coast {
                if visited[q] {
                    break;
                }
                visited[q] = true;
                let downslope = self.corners[q].downslope;
                if q == downslope {
                    break;
                }
                if let Some(edge_id) = self.lookup_edge_from_corner(q, downslope) {
                    self.edges[edge_id].river += 1;
                    self.corners[q].river += 1;
                    self.corners[downslope].river += 1;
                }
                q = downslope;
            }
        }
    }

    fn lookup_edge_from_corner(&self, q: usize, s: usize) -> Option<usize> {
        self.edge_by_corners.get(&sorted_pair(q, s)).copied()
    }

    fn assign_corner_moisture(&mut self) {
        let mut queue = VecDeque::new();
        for q in 0..self.corners.len() {
            if (self.corners[q].water || self.corners[q].river > 0) && !self.corners[q].ocean {
                self.corners[q].moisture = if self.corners[q].river > 0 {
                    (0.2 * self.corners[q].river as f32).min(3.0)
                } else {
                    1.0
                };
                queue.push_back(q);
            } else {
                self.corners[q].moisture = 0.0;
            }
        }

        while let Some(q) = queue.pop_front() {
            let adjacent = self.corners[q].adjacent.clone();
            for r in adjacent {
                let new_moisture = self.corners[q].moisture * 0.9;
                if new_moisture > self.corners[r].moisture {
                    self.corners[r].moisture = new_moisture;
                    queue.push_back(r);
                }
            }
        }

        for corner in &mut self.corners {
            if corner.ocean || corner.coast {
                corner.moisture = 1.0;
            }
        }
    }

    fn redistribute_moisture(&mut self) {
        let mut locations: Vec<usize> = self
            .corners
            .iter()
            .filter(|corner| !corner.ocean && !corner.coast)
            .map(|corner| corner.index)
            .collect();
        locations.sort_by(|&a, &b| {
            self.corners[a]
                .moisture
                .total_cmp(&self.corners[b].moisture)
        });
        if locations.len() > 1 {
            for (i, &corner_id) in locations.iter().enumerate() {
                self.corners[corner_id].moisture = i as f32 / (locations.len() - 1) as f32;
            }
        }
    }

    fn assign_polygon_moisture(&mut self) {
        for center in &mut self.centers {
            center.moisture = average(center.corners.iter().map(|&corner_id| {
                self.corners[corner_id].moisture = self.corners[corner_id].moisture.min(1.0);
                self.corners[corner_id].moisture
            }));
        }
    }

    fn assign_biomes(&mut self) {
        for center in &mut self.centers {
            center.biome = get_biome(center);
        }
    }

    fn create_center_watersheds(&mut self) {
        self.center_watersheds = vec![None; self.centers.len()];
        for center in &self.centers {
            let mut lowest: Option<usize> = None;
            for &corner_id in &center.corners {
                if lowest
                    .map(|candidate| {
                        self.corners[corner_id].elevation < self.corners[candidate].elevation
                    })
                    .unwrap_or(true)
                {
                    lowest = Some(corner_id);
                }
            }
            self.center_watersheds[center.index] =
                lowest.map(|corner_id| self.corners[corner_id].watershed);
        }
    }

    fn build_noisy_edges(&mut self) {
        self.noisy_edges = vec![NoisyEdge::default(); self.edges.len()];
        for center in &self.centers {
            for &edge_id in &center.borders {
                let edge = &self.edges[edge_id];
                let (Some(d0), Some(d1), Some(v0), Some(v1)) = (edge.d0, edge.d1, edge.v0, edge.v1)
                else {
                    continue;
                };
                if self.noisy_edges[edge.index].path0.is_some() {
                    continue;
                }

                let f = 0.5;
                let v0_point = self.corners[v0].point;
                let v1_point = self.corners[v1].point;
                let d0_point = self.centers[d0].point;
                let d1_point = self.centers[d1].point;
                let t = flash_interpolate(v0_point, d0_point, f);
                let q = flash_interpolate(v0_point, d1_point, f);
                let r = flash_interpolate(v1_point, d0_point, f);
                let s = flash_interpolate(v1_point, d1_point, f);

                let mut min_length = 10.0;
                if self.centers[d0].biome != self.centers[d1].biome {
                    min_length = 3.0;
                }
                if self.centers[d0].ocean && self.centers[d1].ocean {
                    min_length = 100.0;
                }
                if self.centers[d0].coast || self.centers[d1].coast || edge.river > 0 {
                    min_length = 1.0;
                }

                self.noisy_edges[edge.index].path0 = Some(build_noisy_line_segments(
                    &mut self.map_random,
                    v0_point,
                    t,
                    edge.midpoint,
                    q,
                    min_length,
                ));
                self.noisy_edges[edge.index].path1 = Some(build_noisy_line_segments(
                    &mut self.map_random,
                    v1_point,
                    r,
                    edge.midpoint,
                    s,
                    min_length,
                ));
            }
        }
    }

    pub(super) fn triangle_color(&self, mode: ViewMode, center_id: usize, edge_id: usize) -> u32 {
        let center = &self.centers[center_id];
        let base = biome_color(center.biome);
        match mode {
            ViewMode::Biome => base,
            ViewMode::Slopes => self.color_with_slope(base, center_id, edge_id),
        }
    }

    fn color_with_slope(&self, color: u32, center_id: usize, edge_id: usize) -> u32 {
        let edge = &self.edges[edge_id];
        let center = &self.centers[center_id];
        let (Some(v0), Some(v1)) = (edge.v0, edge.v1) else {
            return 0x44447a;
        };
        if center.water {
            return color;
        }
        let mut blended = color;
        if let Some(neighbor) = self.other_center(edge_id, center_id)
            && center.water == self.centers[neighbor].water
        {
            blended = interpolate_color(color, biome_color(self.centers[neighbor].biome), 0.4);
        }

        let light = calculate_lighting(
            center.point,
            center.elevation,
            self.corners[v0].point,
            self.corners[v0].elevation,
            self.corners[v1].point,
            self.corners[v1].elevation,
        );
        let color_low = interpolate_color(blended, 0x333333, 0.7);
        let color_high = interpolate_color(blended, 0xffffff, 0.3);
        if light < 0.5 {
            interpolate_color(color_low, blended, light * 2.0)
        } else {
            interpolate_color(blended, color_high, light * 2.0 - 1.0)
        }
    }

    fn other_center(&self, edge_id: usize, center_id: usize) -> Option<usize> {
        let edge = &self.edges[edge_id];
        if edge.d0 == Some(center_id) {
            edge.d1
        } else if edge.d1 == Some(center_id) {
            edge.d0
        } else {
            None
        }
    }

    pub(super) fn biome_counts(&self) -> Vec<BiomeCount> {
        BIOME_DISPLAY_ORDER
            .iter()
            .filter_map(|&(biome, name)| {
                let count = self
                    .centers
                    .iter()
                    .filter(|center| center.biome == biome)
                    .count();
                (count > 0).then_some(BiomeCount {
                    name,
                    color: biome_color(biome),
                    count,
                })
            })
            .collect()
    }
}

const BIOME_DISPLAY_ORDER: [(&str, &str); 19] = [
    ("DEEP_OCEAN", "Deep Ocean"),
    ("SHALLOW_OCEAN", "Shallow Ocean"),
    ("BEACH", "Beach"),
    ("LAKE", "Lake"),
    ("ICE", "Ice"),
    ("MARSH", "Marsh"),
    ("SNOW", "Snow"),
    ("TUNDRA", "Tundra"),
    ("BARE", "Bare"),
    ("SCORCHED", "Scorched"),
    ("TAIGA", "Taiga"),
    ("SHRUBLAND", "Shrubland"),
    ("TEMPERATE_DESERT", "Temperate Desert"),
    ("TEMPERATE_RAIN_FOREST", "Temperate Rain Forest"),
    ("TEMPERATE_DECIDUOUS_FOREST", "Temperate Deciduous Forest"),
    ("GRASSLAND", "Grassland"),
    ("SUBTROPICAL_DESERT", "Subtropical Desert"),
    ("TROPICAL_RAIN_FOREST", "Tropical Rain Forest"),
    ("TROPICAL_SEASONAL_FOREST", "Tropical Seasonal Forest"),
];

fn select_points(point_type: PointType, point_count: usize, _seed: u32) -> Vec<Vec2> {
    match point_type {
        PointType::Square => generate_square_points(point_count),
    }
}

pub(super) fn generate_square_points(point_count: usize) -> Vec<Vec2> {
    let n = (point_count as f32).sqrt() as usize;
    let mut points = Vec::with_capacity(n * n);
    for x in 0..n {
        for y in 0..n {
            points.push(vec2(
                (0.5 + x as f32) / n as f32 * MAP_SIZE,
                (0.5 + y as f32) / n as f32 * MAP_SIZE,
            ));
        }
    }
    points
}

fn build_regions(point_count: usize) -> Vec<Vec<Vec2>> {
    build_square_regions(point_count)
}

fn build_square_regions(point_count: usize) -> Vec<Vec<Vec2>> {
    let n = (point_count as f32).sqrt() as usize;
    let cell = MAP_SIZE / n as f32;
    let mut regions = Vec::with_capacity(n * n);
    for x in 0..n {
        for y in 0..n {
            let left = x as f32 * cell;
            let right = (x + 1) as f32 * cell;
            let top = y as f32 * cell;
            let bottom = (y + 1) as f32 * cell;
            regions.push(vec![
                vec2(left, top),
                vec2(right, top),
                vec2(right, bottom),
                vec2(left, bottom),
            ]);
        }
    }
    regions
}

fn make_corner(
    corners: &mut Vec<Corner>,
    lookup: &mut HashMap<(i32, i32), usize>,
    point: Vec2,
) -> usize {
    let clamped = vec2(point.x.clamp(0.0, MAP_SIZE), point.y.clamp(0.0, MAP_SIZE));
    let key = (
        (clamped.x * 1000.0).round() as i32,
        (clamped.y * 1000.0).round() as i32,
    );
    if let Some(&corner_id) = lookup.get(&key) {
        return corner_id;
    }
    let border = clamped.x <= 0.001
        || clamped.x >= MAP_SIZE - 0.001
        || clamped.y <= 0.001
        || clamped.y >= MAP_SIZE - 0.001;
    let index = corners.len();
    corners.push(Corner {
        index,
        point: clamped,
        ocean: false,
        water: false,
        coast: false,
        border,
        elevation: 0.0,
        moisture: 0.0,
        touches: Vec::new(),
        protrudes: Vec::new(),
        adjacent: Vec::new(),
        river: 0,
        downslope: index,
        watershed: index,
        watershed_size: 0,
    });
    lookup.insert(key, index);
    index
}

fn improve_corners(centers: &mut [Center], corners: &mut [Corner], edges: &mut [Edge]) {
    let new_points: Vec<Vec2> = corners
        .iter()
        .map(|corner| {
            if corner.border {
                corner.point
            } else {
                corner
                    .touches
                    .iter()
                    .fold(Vec2::ZERO, |acc, &center_id| acc + centers[center_id].point)
                    / corner.touches.len() as f32
            }
        })
        .collect();

    for (corner, point) in corners.iter_mut().zip(new_points) {
        corner.point = point;
    }
    for edge in edges {
        if let (Some(v0), Some(v1)) = (edge.v0, edge.v1) {
            edge.midpoint = (corners[v0].point + corners[v1].point) * 0.5;
        }
    }
}

fn shallow_ocean_jitter(point: Vec2) -> f32 {
    let x = (point.x / MAP_SIZE * 17.0).floor() as i32;
    let y = (point.y / MAP_SIZE * 17.0).floor() as i32;
    let mut hash = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    hash ^= hash >> 16;
    hash = hash.wrapping_mul(0x7feb_352d);
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(0x846c_a68b);
    (hash ^ (hash >> 16)) as f32 / u32::MAX as f32
}

fn push_unique<T: PartialEq>(items: &mut Vec<T>, item: T) {
    if !items.contains(&item) {
        items.push(item);
    }
}

fn sorted_pair(a: usize, b: usize) -> (usize, usize) {
    if a < b { (a, b) } else { (b, a) }
}

fn average(values: impl Iterator<Item = f32>) -> f32 {
    let mut total = 0.0;
    let mut count = 0usize;
    for value in values {
        total += value;
        count += 1;
    }
    if count == 0 {
        0.0
    } else {
        total / count as f32
    }
}

fn flash_interpolate(a: Vec2, b: Vec2, f: f32) -> Vec2 {
    b + (a - b) * f
}

fn build_noisy_line_segments(
    rng: &mut MapRng,
    a: Vec2,
    b: Vec2,
    c: Vec2,
    d: Vec2,
    min_length: f32,
) -> Vec<Vec2> {
    fn subdivide(
        rng: &mut MapRng,
        points: &mut Vec<Vec2>,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        d: Vec2,
        min_length: f32,
    ) {
        if a.distance(c) < min_length || b.distance(d) < min_length {
            return;
        }

        let p = map_random_f32(rng, 0.2..0.8);
        let q = map_random_f32(rng, 0.2..0.8);
        let e = flash_interpolate(a, d, p);
        let f = flash_interpolate(b, c, p);
        let g = flash_interpolate(a, b, q);
        let i = flash_interpolate(d, c, q);
        let h = flash_interpolate(e, f, q);
        let s = 1.0 - map_random_f32(rng, -0.4..0.4);
        let t = 1.0 - map_random_f32(rng, -0.4..0.4);

        subdivide(
            rng,
            points,
            a,
            flash_interpolate(g, b, s),
            h,
            flash_interpolate(e, d, t),
            min_length,
        );
        points.push(h);
        subdivide(
            rng,
            points,
            h,
            flash_interpolate(f, c, s),
            c,
            flash_interpolate(i, d, t),
            min_length,
        );
    }

    let mut points = vec![a];
    subdivide(rng, &mut points, a, b, c, d, min_length);
    points.push(c);
    points
}
