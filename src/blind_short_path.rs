use tfhe::core_crypto::prelude::{lwe_ciphertext_add, lwe_ciphertext_add_assign};

use crate::{Context, LUT, LWE};

impl crate::PublicKey {
    pub fn blind_sp(
        &self,
        next_mat: &Vec<Vec<u64>>,
        dep: LWE,
        arv: LWE,
        ctx: &Context,
    ) -> Vec<LWE> {
        let n = next_mat.len();
        let colonne = self.blind_extract_vec_clear(&next_mat, &arv, ctx);
        let mut vecteur = vec![dep.clone()];
        for i in 1..n {
            vecteur.push(self.blind_array_access(&vecteur[i - 1], &colonne, ctx));
        }

        vecteur
    }
}
#[cfg(test)]
mod tests {
    use tfhe::shortint::parameters::{*};

    use crate::{key, Context};

    #[test]
    pub fn test_sp() {
        let mut ctx = Context::from(PARAM_MESSAGE_5_CARRY_0);
        let private_key = key(ctx.parameters);
        let public_key = &private_key.public_key;

        let next = vec![
            vec![0, 14, 2, 14, 2, 7, 6, 7, 8, 14, 8, 8, 14, 14, 14, 7],
            vec![0, 1, 12, 3, 12, 5, 12, 12, 3, 14, 12, 11, 12, 12, 14, 12],
            vec![7, 4, 2, 4, 4, 7, 7, 7, 7, 4, 10, 11, 4, 4, 4, 7],
            vec![0, 1, 12, 3, 4, 12, 12, 12, 8, 12, 12, 12, 12, 12, 14, 12],
            vec![1, 1, 2, 1, 4, 5, 6, 2, 1, 9, 1, 1, 1, 1, 14, 2],
            vec![8, 10, 8, 10, 10, 5, 8, 8, 8, 14, 10, 10, 10, 14, 14, 8],
            vec![7, 2, 2, 2, 2, 7, 6, 7, 7, 14, 2, 2, 2, 13, 14, 7],
            vec![0, 6, 6, 6, 6, 5, 6, 7, 15, 15, 5, 6, 6, 15, 15, 15],
            vec![0, 2, 2, 10, 2, 2, 2, 2, 8, 14, 10, 10, 10, 14, 14, 15],
            vec![0, 1, 2, 1, 2, 5, 2, 2, 0, 9, 12, 1, 12, 12, 1, 2],
            vec![0, 4, 2, 11, 4, 5, 11, 11, 0, 11, 10, 11, 12, 12, 0, 11],
            vec![1, 1, 2, 3, 2, 7, 7, 7, 7, 9, 7, 11, 12, 12, 1, 7],
            vec![10, 13, 2, 13, 4, 5, 7, 7, 15, 13, 10, 10, 12, 13, 15, 15],
            vec![0, 1, 7, 1, 7, 7, 7, 7, 7, 9, 7, 1, 1, 13, 7, 7],
            vec![9, 9, 9, 9, 9, 13, 13, 13, 13, 9, 10, 9, 9, 13, 14, 13],
            vec![8, 4, 8, 4, 4, 5, 7, 7, 8, 14, 8, 8, 14, 14, 14, 15],
        ];

        let dep = private_key.allocate_and_encrypt_lwe(5, &mut ctx);
        let arr = private_key.allocate_and_encrypt_lwe(12, &mut ctx);

        let sp = public_key.blind_sp(&next, dep, arr, &ctx);
        for LWE in sp {
            private_key.debug_lwe("hello", &LWE, &ctx);
        }
    }
}
