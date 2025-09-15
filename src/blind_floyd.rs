use tfhe::core_crypto::prelude::{lwe_ciphertext_add, lwe_ciphertext_add_assign};

use crate::{Context, LUT, LWE};

impl crate::PublicKey {
    pub fn blind_floyd_warshall(
        &self,
        adj_mat_enc: Vec<Vec<LWE>>,
        next_step: Vec<Vec<LWE>>,
        context: &Context,
    ) -> (Vec<Vec<LWE>>, Vec<Vec<LWE>>) {
        let n = adj_mat_enc.len();
        let mut A = adj_mat_enc.clone();
        let mut  N = next_step.clone();
        
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    //blind_add_infini prend 17 rotation parcequelle appelle blind_matrix_acess
                    let s = self.blind_add_infini(&A[i][k], &A[k][j], &context);
                    //blindltbmamv prend 3 rotation
                    let b = self.blind_lt_bma_mv(&s, &A[i][j], &context);
                    // ici on peut gqgner un lookup entre celui du bmamv et celui qu on refqit qpres
                    let l = LUT::from_vec_of_lwe(&vec![A[i][j].clone(), s], &self, &context);
                    //baa prend une rotation
                    A[i][j] = self.blind_array_access(&b, &l, &context);
                    
                    let l2= LUT::from_vec_of_lwe(&vec![N[i][j].clone(), N[i][k].clone()], &self, &context);
                    
                    N[i][j]= self.blind_array_access(&b, &l2, &context); 
                }
            }
            
        }
        (A,N)
    }

   

}



#[cfg(test)]
mod tests {
    use std::time::Instant;

    use tfhe::shortint::parameters::PARAM_MESSAGE_4_CARRY_0;

    use crate::{key, Context};

    #[test]
    pub fn test_floyd() {
        let mut context = Context::from(PARAM_MESSAGE_4_CARRY_0);
        let private_key = key(context.parameters);
        let public_key = &private_key.public_key;
        let mat: Vec<Vec<u64>> = vec![
            vec![0, 3, 15, 7, 15],   
            vec![15, 0, 1, 15, 15],  
            vec![15, 15, 0, 2, 15],  
            vec![15, 15, 15, 0, 3],  
            vec![15, 15, 15, 15, 0],
        ];
        let next: Vec<Vec<u64>> = vec![
            vec![0, 1, 15, 3, 15],   
            vec![15, 1, 2, 15, 15],  
            vec![15, 15, 2, 3, 15],  
            vec![15, 15, 15, 3, 4],  
            vec![15, 15, 15, 15, 4],
        ];
        
        //mat.iter().map(|lin| lin.iter().map(|elt| {private_key.allocate_and_encrypt_lwe(*elt, &mut context)}));
        let mut enc_mat = vec![];
        for line in mat {
            let mut enc_lin = vec![];
            for element in line {
                let enc_elt = private_key.allocate_and_encrypt_lwe(element, &mut context);
                enc_lin.push(enc_elt);
            }
            enc_mat.push(enc_lin);
        }

        let mut enc_next = vec![];
        for line in next {
            let mut enc_line = vec![];
            for element in line {
                let enc_elt = private_key.allocate_and_encrypt_lwe(element, &mut context);
                enc_line.push(enc_elt);
            }
            enc_next.push(enc_line);
        }

        let start=Instant::now();
        let (res_mat, res_next) = public_key.blind_floyd_warshall(enc_mat, enc_next, &context);
        
        
        let decrypt_res : Vec<Vec<u64>>= res_mat.iter().map(|lin| {
            lin.iter()
                .map(|elt| private_key.decrypt_lwe(&elt, &context))
                .collect()
        }).collect();

        let decrypt_res_next : Vec<Vec<u64>>= res_next.iter().map(|lin| {
            lin.iter()
                .map(|elt| private_key.decrypt_lwe(&elt, &context))
                .collect()
        }).collect();

        let duration=start.elapsed();

       

        println!("{:?}",decrypt_res);
        print!("{:?}",decrypt_res_next);
        print!("{:?}",duration);
    }
}
