use std::f64::consts::PI;

/// Struct utama untuk perhitungan debit air
#[derive(Debug, Clone)]
pub struct DebitAir {
    /// Debit dalam m³/s
    pub debit: f64,
    /// Kecepatan aliran dalam m/s
    pub kecepatan: f64,
    /// Luas penampang dalam m²
    pub luas_penampang: f64,
}

/// Metode perhitungan yang tersedia
#[derive(Debug, Clone, Copy)]
pub enum MetodePerhitungan {
    /// Q = V × A (Kecepatan × Luas Penampang)
    KecepatanLuas,
    /// Q = Cd × A × √(2gh) (Untuk lubang/orifis)
    Orifis,
    /// Q = (1/n) × A × R^(2/3) × S^(1/2) (Manning)
    Manning,
    /// Q = C × A × √(2gH) (Weir/Bendung)
    Weir,
}

/// Tipe penampang aliran
#[derive(Debug, Clone, Copy)]
pub enum TipePenampang {
    Persegi,
    Lingkaran,
    Trapesium,
    Segitiga,
}

/// Parameter untuk penampang
#[derive(Debug, Clone)]
pub struct ParameterPenampang {
    pub tipe: TipePenampang,
    pub lebar: f64,        // Untuk persegi & trapesium (m)
    pub tinggi: f64,       // Untuk persegi & trapesium (m)
    pub diameter: f64,     // Untuk lingkaran (m)
    pub lebar_atas: f64,   // Untuk trapesium (m)
    pub sudut: f64,        // Untuk trapesium & segitiga (radian)
}

/// Parameter untuk metode Manning
#[derive(Debug, Clone)]
pub struct ParameterManning {
    pub koefisien_kekasaran: f64, // n (Manning)
    pub kemiringan_dasar: f64,    // S
    pub jari_jari_hidrolis: f64,  // R
}

/// Parameter untuk metode Orifis
#[derive(Debug, Clone)]
pub struct ParameterOrifis {
    pub koefisien_discharge: f64, // Cd
    pub percepatan_gravitasi: f64, // g (m/s²)
    pub tinggi_tekanan: f64,      // h (m)
}

/// Parameter untuk Weir/Bendung
#[derive(Debug, Clone)]
pub struct ParameterWeir {
    pub koefisien_weir: f64,      // C
    pub lebar_bendung: f64,       // L (m)
    pub tinggi_tekanan: f64,      // H (m)
    pub percepatan_gravitasi: f64, // g (m/s²)
}

/// Error handling untuk perhitungan
#[derive(Debug, thiserror::Error)]
pub enum DebitError {
    #[error("Nilai tidak boleh negatif: {0}")]
    NilaiNegatif(String),
    #[error("Parameter tidak valid: {0}")]
    ParameterInvalid(String),
    #[error("Pembagian dengan nol")]
    DivisionByZero,
}

/// Main struct untuk kalkulator debit
pub struct KalkulatorDebit {
    /// Metode perhitungan yang digunakan
    pub metode: MetodePerhitungan,
    /// Presisi desimal output
    pub presisi: u32,
}

impl Default for KalkulatorDebit {
    fn default() -> Self {
        Self {
            metode: MetodePerhitungan::KecepatanLuas,
            presisi: 3,
        }
    }
}

impl DebitAir {
    /// Membuat instance DebitAir baru
    pub fn baru(kecepatan: f64, luas_penampang: f64) -> Result<Self, DebitError> {
        if kecepatan < 0.0 {
            return Err(DebitError::NilaiNegatif("kecepatan".to_string()));
        }
        if luas_penampang < 0.0 {
            return Err(DebitError::NilaiNegatif("luas_penampang".to_string()));
        }
        
        Ok(Self {
            debit: kecepatan * luas_penampang,
            kecepatan,
            luas_penampang,
        })
    }

    /// Menghitung debit dari metode Kecepatan-Luas
    pub fn dari_kecepatan_luas(kecepatan: f64, luas: f64) -> Result<Self, DebitError> {
        Self::baru(kecepatan, luas)
    }

    /// Menghitung debit dari metode Orifis
    pub fn dari_orifis(param: &ParameterOrifis, luas: f64) -> Result<Self, DebitError> {
        if param.koefisien_discharge <= 0.0 || param.koefisien_discharge > 1.0 {
            return Err(DebitError::ParameterInvalid(
                "Koefisien discharge harus antara 0-1".to_string()
            ));
        }
        if param.tinggi_tekanan < 0.0 {
            return Err(DebitError::NilaiNegatif("tinggi_tekanan".to_string()));
        }
        if luas <= 0.0 {
            return Err(DebitError::NilaiNegatif("luas".to_string()));
        }

        let kecepatan = param.koefisien_discharge * (2.0 * param.percepatan_gravitasi * param.tinggi_tekanan).sqrt();
        let debit = param.koefisien_discharge * luas * (2.0 * param.percepatan_gravitasi * param.tinggi_tekanan).sqrt();
        
        Ok(Self {
            debit,
            kecepatan,
            luas_penampang: luas,
        })
    }

    /// Menghitung debit dari metode Manning
    pub fn dari_manning(param: &ParameterManning, luas: f64) -> Result<Self, DebitError> {
        if param.koefisien_kekasaran <= 0.0 {
            return Err(DebitError::NilaiNegatif("koefisien_kekasaran".to_string()));
        }
        if param.jari_jari_hidrolis <= 0.0 {
            return Err(DebitError::NilaiNegatif("jari_jari_hidrolis".to_string()));
        }
        if param.kemiringan_dasar < 0.0 {
            return Err(DebitError::NilaiNegatif("kemiringan_dasar".to_string()));
        }
        if luas <= 0.0 {
            return Err(DebitError::NilaiNegatif("luas".to_string()));
        }

        let kecepatan = (1.0 / param.koefisien_kekasaran) * 
                       param.jari_jari_hidrolis.powf(2.0/3.0) * 
                       param.kemiringan_dasar.sqrt();
        let debit = kecepatan * luas;
        
        Ok(Self {
            debit,
            kecepatan,
            luas_penampang: luas,
        })
    }

    /// Menghitung debit dari metode Weir
    pub fn dari_weir(param: &ParameterWeir) -> Result<Self, DebitError> {
        if param.koefisien_weir <= 0.0 {
            return Err(DebitError::NilaiNegatif("koefisien_weir".to_string()));
        }
        if param.lebar_bendung <= 0.0 {
            return Err(DebitError::NilaiNegatif("lebar_bendung".to_string()));
        }
        if param.tinggi_tekanan < 0.0 {
            return Err(DebitError::NilaiNegatif("tinggi_tekanan".to_string()));
        }

        let debit = param.koefisien_weir * param.lebar_bendung * 
                   (2.0 * param.percepatan_gravitasi).sqrt() * 
                   param.tinggi_tekanan.powf(1.5);
        
        // Kecepatan rata-rata (pendekatan)
        let luas = param.lebar_bendung * param.tinggi_tekanan;
        let kecepatan = debit / luas;
        
        Ok(Self {
            debit,
            kecepatan,
            luas_penampang: luas,
        })
    }

    /// Konversi ke liter/detik
    pub fn ke_liter_per_detik(&self) -> f64 {
        self.debit * 1000.0
    }

    /// Konversi ke m³/jam
    pub fn ke_m3_per_jam(&self) -> f64 {
        self.debit * 3600.0
    }

    /// Konversi ke liter/menit
    pub fn ke_liter_per_menit(&self) -> f64 {
        self.debit * 1000.0 * 60.0
    }

    /// Menampilkan hasil dengan formatting
    pub fn tampilkan(&self, presisi: u32) -> String {
        format!(
            "Debit: {:.prec$} m³/s\n\
             Kecepatan: {:.prec$} m/s\n\
             Luas Penampang: {:.prec$} m²\n\
             = {:.prec$} L/s\n\
             = {:.prec$} m³/jam\n\
             = {:.prec$} L/menit",
            self.debit,
            self.kecepatan,
            self.luas_penampang,
            self.ke_liter_per_detik(),
            self.ke_m3_per_jam(),
            self.ke_liter_per_menit(),
            prec = presisi as usize
        )
    }
}

impl KalkulatorDebit {
    /// Membuat kalkulator baru
    pub fn baru() -> Self {
        Self::default()
    }

    /// Mengatur metode perhitungan
    pub fn dengan_metode(mut self, metode: MetodePerhitungan) -> Self {
        self.metode = metode;
        self
    }

    /// Mengatur presisi
    pub fn dengan_presisi(mut self, presisi: u32) -> Self {
        self.presisi = presisi;
        self
    }

    /// Menghitung luas penampang
    pub fn hitung_luas(&self, param: &ParameterPenampang) -> Result<f64, DebitError> {
        match param.tipe {
            TipePenampang::Persegi => {
                if param.lebar <= 0.0 || param.tinggi <= 0.0 {
                    return Err(DebitError::NilaiNegatif("lebar/tinggi".to_string()));
                }
                Ok(param.lebar * param.tinggi)
            }
            TipePenampang::Lingkaran => {
                if param.diameter <= 0.0 {
                    return Err(DebitError::NilaiNegatif("diameter".to_string()));
                }
                let r = param.diameter / 2.0;
                Ok(PI * r * r)
            }
            TipePenampang::Trapesium => {
                if param.lebar <= 0.0 || param.lebar_atas <= 0.0 || param.tinggi <= 0.0 {
                    return Err(DebitError::NilaiNegatif("parameter trapesium".to_string()));
                }
                Ok((param.lebar + param.lebar_atas) / 2.0 * param.tinggi)
            }
            TipePenampang::Segitiga => {
                if param.lebar <= 0.0 || param.tinggi <= 0.0 {
                    return Err(DebitError::NilaiNegatif("parameter segitiga".to_string()));
                }
                Ok(0.5 * param.lebar * param.tinggi)
            }
        }
    }

    /// Menghitung debit dengan metode yang dipilih
    pub fn hitung(
        &self,
        luas: f64,
        params: Option<&dyn std::any::Any>,
    ) -> Result<DebitAir, DebitError> {
        match self.metode {
            MetodePerhitungan::KecepatanLuas => {
                if luas <= 0.0 {
                    return Err(DebitError::NilaiNegatif("luas".to_string()));
                }
                // Untuk metode ini, parameter adalah kecepatan
                let kecepatan = params
                    .and_then(|p| p.downcast_ref::<f64>())
                    .ok_or_else(|| DebitError::ParameterInvalid(
                        "Parameter kecepatan diperlukan".to_string()
                    ))?;
                DebitAir::dari_kecepatan_luas(*kecepatan, luas)
            }
            MetodePerhitungan::Orifis => {
                let param = params
                    .and_then(|p| p.downcast_ref::<ParameterOrifis>())
                    .ok_or_else(|| DebitError::ParameterInvalid(
                        "Parameter orifis diperlukan".to_string()
                    ))?;
                DebitAir::dari_orifis(param, luas)
            }
            MetodePerhitungan::Manning => {
                let param = params
                    .and_then(|p| p.downcast_ref::<ParameterManning>())
                    .ok_or_else(|| DebitError::ParameterInvalid(
                        "Parameter manning diperlukan".to_string()
                    ))?;
                DebitAir::dari_manning(param, luas)
            }
            MetodePerhitungan::Weir => {
                let param = params
                    .and_then(|p| p.downcast_ref::<ParameterWeir>())
                    .ok_or_else(|| DebitError::ParameterInvalid(
                        "Parameter weir diperlukan".to_string()
                    ))?;
                DebitAir::dari_weir(param)
            }
        }
    }
}

// Contoh penggunaan
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debit_kecepatan_luas() {
        let debit = DebitAir::dari_kecepatan_luas(2.0, 3.0).unwrap();
        assert_eq!(debit.debit, 6.0);
        assert_eq!(debit.ke_liter_per_detik(), 6000.0);
    }

    #[test]
    fn test_debit_manning() {
        let param = ParameterManning {
            koefisien_kekasaran: 0.03,
            kemiringan_dasar: 0.001,
            jari_jari_hidrolis: 1.5,
        };
        let debit = DebitAir::dari_manning(&param, 4.0).unwrap();
        assert!(debit.debit > 0.0);
    }

    #[test]
    fn test_hitung_luas_persegi() {
        let kalkulator = KalkulatorDebit::baru();
        let param = ParameterPenampang {
            tipe: TipePenampang::Persegi,
            lebar: 2.0,
            tinggi: 3.0,
            diameter: 0.0,
            lebar_atas: 0.0,
            sudut: 0.0,
        };
        let luas = kalkulator.hitung_luas(&param).unwrap();
        assert_eq!(luas, 6.0);
    }
}

// Fungsi main untuk contoh penggunaan
fn main() -> Result<(), DebitError> {
    println!("=== SISTEM PERHITUNGAN DEBIT AIR ===\n");

    // Contoh 1: Metode Kecepatan-Luas
    println!("1. Metode Kecepatan × Luas Penampang:");
    let debit1 = DebitAir::dari_kecepatan_luas(2.5, 1.8)?;
    println!("{}\n", debit1.tampilkan(3));

    // Contoh 2: Metode Manning
    println!("2. Metode Manning (Saluran Terbuka):");
    let manning_param = ParameterManning {
        koefisien_kekasaran: 0.03,
        kemiringan_dasar: 0.001,
        jari_jari_hidrolis: 1.5,
    };
    let debit2 = DebitAir::dari_manning(&manning_param, 4.0)?;
    println!("{}\n", debit2.tampilkan(3));

    // Contoh 3: Metode Orifis
    println!("3. Metode Orifis (Lubang):");
    let orifis_param = ParameterOrifis {
        koefisien_discharge: 0.62,
        percepatan_gravitasi: 9.81,
        tinggi_tekanan: 2.0,
    };
    let debit3 = DebitAir::dari_orifis(&orifis_param, 0.5)?;
    println!("{}\n", debit3.tampilkan(3));

    // Contoh 4: Metode Weir
    println!("4. Metode Weir (Bendung):");
    let weir_param = ParameterWeir {
        koefisien_weir: 1.7,
        lebar_bendung: 2.0,
        tinggi_tekanan: 0.8,
        percepatan_gravitasi: 9.81,
    };
    let debit4 = DebitAir::dari_weir(&weir_param)?;
    println!("{}\n", debit4.tampilkan(3));

    // Contoh 5: Menggunakan Kalkulator dengan berbagai metode
    println!("5. Menggunakan Kalkulator:");
    let kalkulator = KalkulatorDebit::baru()
        .dengan_metode(MetodePerhitungan::KecepatanLuas)
        .dengan_presisi(4);

    let param_penampang = ParameterPenampang {
        tipe: TipePenampang::Lingkaran,
        lebar: 0.0,
        tinggi: 0.0,
        diameter: 0.5,
        lebar_atas: 0.0,
        sudut: 0.0,
    };

    let luas = kalkulator.hitung_luas(&param_penampang)?;
    let kecepatan = 1.2;
    let debit5 = kalkulator.hitung(luas, Some(&kecepatan as &dyn std::any::Any))?;
    println!("Penampang Lingkaran (d=0.5m) dengan v=1.2m/s:");
    println!("{}\n", debit5.tampilkan(4));

    // Contoh konversi satuan
    println!("6. Konversi Satuan:");
    let debit = DebitAir::baru(1.0, 1.0)?;
    println!("1 m³/s = {} L/s", debit.ke_liter_per_detik());
    println!("1 m³/s = {} m³/jam", debit.ke_m3_per_jam());
    println!("1 m³/s = {} L/menit", debit.ke_liter_per_menit());

    Ok(())
}
