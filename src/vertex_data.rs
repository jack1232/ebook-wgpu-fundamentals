use cgmath::*;

pub fn torus_position(r_torus:f32, r_tube:f32, u:Deg<f32>, v: Deg<f32>) -> [f32; 3] {
    let x = (r_torus + r_tube * v.cos()) * u.cos();
    let y = r_tube * v.sin();
    let z = -(r_torus + r_tube * v.cos()) * u.sin();
    [x, y, z]
}

pub fn create_torus_data(r_torus:f32, r_tube:f32, n_torus:u16, n_tube:u16) 
-> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u16>, Vec<u16>) {
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let eps = 0.01 * 360.0/n_tube as f32;
    
    for i in 0..=n_torus {
        let du = i as f32 * 360.0/n_torus as f32;
        for j in 0..=n_tube {
            let dv = j as f32 * 360.0/n_tube as f32;
            let pos = torus_position(r_torus, r_tube, Deg(du), Deg(dv));
            positions.push(pos);

            // calculate normals
            let nu = Vector3::from(torus_position(r_torus, r_tube, Deg(du+eps), Deg(dv))) -
                     Vector3::from(torus_position(r_torus, r_tube, Deg(du-eps), Deg(dv)));
            let nv = Vector3::from(torus_position(r_torus, r_tube, Deg(du), Deg(dv+eps))) -
                     Vector3::from(torus_position(r_torus, r_tube, Deg(du), Deg(dv-eps)));
            let normal = nu.cross(nv).normalize();
            normals.push(normal.into());
        }
    }

    let mut indices: Vec<u16> = vec![];
    let mut indices2: Vec<u16> = vec![];
    let vertices_per_row = n_tube + 1;

    for i in 0..n_torus {
        for j in 0..n_tube {
            let idx0 = j + i * vertices_per_row;
            let idx1 = j + 1 + i * vertices_per_row;
            let idx2 = j + 1 + (i + 1) * vertices_per_row;
            let idx3 = j + (i + 1) * vertices_per_row; 
            let values:Vec<u16> = vec![idx0, idx1, idx2, idx2, idx3, idx0];
            indices.extend(values);
            let values2:Vec<u16> = vec![idx0, idx1, idx0, idx3];
            indices2.extend(values2);
        }
    }

    (positions, normals, indices, indices2)
}


fn cylinder_position(r:f32, theta:Deg<f32>, y:f32) -> [f32; 3] {
    let x = r * theta.cos();
    let z = - r * theta.sin();
    [x, y, z]
}

pub fn create_cylinder_data(mut rin:f32, rout:f32, h:f32, n:u16) -> (Vec<[f32; 3]>, Vec<u16>, Vec<u16>) {
    if rin >= 0.999 * rout { 
        rin = 0.999 * rout; 
    }

    let mut positions: Vec<[f32; 3]> = vec![];
    for i in 0..=n {
        let theta = i as f32 * 360.0/n as f32;
        let p0 = cylinder_position(rout, Deg(theta), h/2.0);
        let p1 = cylinder_position(rout, Deg(theta), -h/2.0);
        let p2 = cylinder_position(rin, Deg(theta), -h/2.0);
        let p3 = cylinder_position(rin, Deg(theta), h/2.0);
        let values:Vec<[f32; 3]> = vec![p0, p1, p2, p3];
        positions.extend(values);
    }

    let mut indices: Vec<u16> = vec![];
    let mut indices2: Vec<u16> = vec![];

    for i in 0..n {
        let idx0 = i*4;
        let idx1 = i*4 + 1;
        let idx2 = i*4 + 2;
        let idx3 = i*4 + 3;
        let idx4 = i*4 + 4;
        let idx5 = i*4 + 5;
        let idx6 = i*4 + 6;
        let idx7 = i*4 + 7;

        // triangle indices
        let values: Vec<u16> = vec![
            idx0, idx4, idx7, idx7, idx3, idx0, // top
            idx1, idx2, idx6, idx6, idx5, idx1, // bottom
            idx0, idx1, idx5, idx5, idx4, idx0, // outer
            idx2, idx3, idx7, idx7, idx6, idx2  // inner
        ];
        indices.extend(values);

        // wireframe indices
        let values2: Vec<u16> = vec![
            idx0, idx3, idx3, idx7, idx4, idx0, // top
            idx1, idx2, idx2, idx6, idx5, idx1, // bottom
            idx0, idx1, idx3, idx2              // side
        ];
        indices2.extend(values2);
    }

    (positions, indices, indices2)
}

fn sphere_position(r:f32, theta:Deg<f32>, phi:Deg<f32>) -> [f32; 3] {
    let x = r * theta.sin() * phi.cos();
    let y = r * theta.cos();
    let z =  -r * theta.sin() * phi.sin();
    [x, y, z]
}

pub fn create_sphere_data(r:f32, u:u16, v:u16) -> 
(Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>, Vec<u16>) {
    let mut positions: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 2]> = vec![];
   
    for i in 0..=u {
        for j in 0..=v {
            let theta = i as f32 *180.0/u as f32;
            let phi = j as f32 * 360.0/v as f32;
            let pos = sphere_position(r, Deg(theta), Deg(phi));
            positions.push(pos);
            normals.push([pos[0]/r, pos[1]/r, pos[2]/r]);
            uvs.push([i as f32/u as f32, j as f32/v as f32]);
        }
    }

    let mut indices: Vec<u16> = vec![];
    let mut indices2: Vec<u16> = vec![];
    
    for i in 0..u {
        for j in 0..v {
            let idx0 = j + i * (v as u16 + 1);
            let idx1 = j + 1 + i * (v as u16 + 1);
            let idx2 = j + 1 + (i + 1) * (v as u16 + 1);
            let idx3 = j + (i + 1) * (v as u16 + 1);

            let values: Vec<u16> = vec![idx0, idx1, idx2, idx2, idx3, idx0];
            indices.extend(values); 
           
            let values2: Vec<u16> = vec![idx0, idx1, idx0, idx3];
            indices2.extend(values2); 
        }
    }

    (positions, normals, uvs, indices, indices2)
}

pub fn create_cube_data(side:f32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>,Vec<[f32; 3]>,
    Vec<[f32; 2]>,Vec<u16>,Vec<u16>) {
        let s2 = side / 2.0;
        let positions = [
            [s2,  s2,  s2],     // index 0
            [s2,  s2, -s2],     // index 1
            [s2, -s2,  s2],     // index 2
            [s2, -s2, -s2],     // index 3
            [-s2,  s2, -s2],    // index 4
            [-s2,  s2,  s2],    // index 5
            [-s2, -s2, -s2],    // index 6
            [-s2, -s2,  s2],    // index 7
            [-s2,  s2, -s2],    // index 8
            [s2,  s2, -s2],     // index 9
            [-s2,  s2,  s2],    // index 10
            [s2,  s2,  s2],     // index 11
            [-s2, -s2,  s2],    // index 12
            [s2, -s2,  s2],     // index 13
            [-s2, -s2, -s2],    // index 14
            [s2, -s2, -s2],     // index 15
            [-s2,  s2,  s2],    // index 16
            [s2,  s2,  s2],     // index 17
            [-s2, -s2,  s2],    // index 18
            [s2, -s2,  s2],     // index 19
            [s2,  s2, -s2],     // index 20
            [-s2,  s2, -s2],    // index 21
            [s2, -s2, -s2],     // index 22
            [-s2, -s2, -s2],    // index 23
        ];
    
        let colors = [
            [1., 1., 1.], [1., 1., 0.], [1., 0., 1.], [1., 0., 0.],
            [0., 1., 0.], [0., 1., 1.], [0., 0., 0.], [0., 0., 1.],
            [0., 1., 0.], [1., 1., 0.], [0., 1., 1.], [1., 1., 1.],
            [0., 0., 1.], [1., 0., 1.], [0., 0., 0.], [1., 0., 0.],
            [0., 1., 1.], [1., 1., 1.], [0., 0., 1.], [1., 0., 1.],
            [1., 1., 0.], [0., 1., 0.], [1., 0., 0.], [0., 0., 0.],
        ];
    
        let normals = [
            [1., 0.,  0.],  [1.,  0.,  0.],  [1.,  0.,  0.],  [1.,  0.,  0.],
            [-1.,  0.,  0.], [-1.,  0.,  0.], [-1.,  0.,  0.], [-1.,  0.,  0.],
            [0.,  1.,  0.],  [0.,  1.,  0.],  [0.,  1.,  0.],  [0.,  1.,  0.],
            [0., -1.,  0.],  [0., -1.,  0.],  [0., -1.,  0.],  [0., -1.,  0.],
            [0.,  0.,  1.],  [0.,  0.,  1.],  [0.,  0.,  1.],  [0.,  0.,  1.],
            [0.,  0., -1.],  [0.,  0., -1.],  [0.,  0., -1.],  [0.,  0., -1.],
        ];
    
        let uvs = [
            [0., 1.], [1., 1.], [0., 0.], [1., 0.], [0., 1.], [1., 1.], [0., 0.], [1., 0.], 
            [0., 1.], [1., 1.], [0., 0.], [1., 0.], [0., 1.], [1., 1.], [0., 0.], [1., 0.], 
            [0., 1.], [1., 1.], [0., 0.], [1., 0.], [0., 1.], [1., 1.], [0., 0.], [1., 0.], 
        ];
    
        let indices = [
            0,  2,  1, 2,  3,  1,
            4,  6,  5, 6,  7,  5,
            8, 10,  9, 10, 11, 9,
            12, 14, 13, 14, 15, 13,
            16, 18, 17, 18, 19, 17,
            20, 22, 21, 22, 23, 21,
        ];
    
        let indices2 = [
            8, 9, 9, 11, 11, 10, 10, 8,     // top
            14, 15, 15, 13, 13, 12, 12, 14, // bottom
            11, 13, 9, 15, 8, 14, 10, 12,   // side
        ];
    (positions.to_vec(), colors.to_vec(), normals.to_vec(), uvs.to_vec(), 
     indices.to_vec(), indices2.to_vec())
}
    