use winapi::um::d3d11::*;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;

#[derive(Debug)]
pub enum DdsParserError {
	InvalidHeader
}

pub fn parse_dds_header( src_data : &[u8] ) -> Result<D3D11_TEXTURE2D_DESC, DdsParserError> {
	Err(DdsParserError::InvalidHeader)
}

#[cfg(test)]
mod tests {
    use super::*;

	// embed the data we will be testing against
	mod paintnet {
		pub static BLACK_4X4_BC1      : &'static [u8; 240128] = include_bytes!("../tests/data/paintnet/black_4x4_bc1.dds");
		// pub static BLACK_4X4_MIPS_BC1 : &'static [u8; 320552] = include_bytes!("../tests/data/paintnet/black_4x4_mips_bc1.dds");
	}

    #[test]
    fn sum_test() {

		let texture_header_ref = D3D11_TEXTURE2D_DESC {
			Width : 4,
			Height : 4,
			MipLevels : 0,
			ArraySize : 0,
			Format : DXGI_FORMAT_BC1_UNORM,
			SampleDesc : DXGI_SAMPLE_DESC  {
				Count : 1,
				Quality : 1,
			},
			Usage : D3D11_USAGE_DEFAULT,
			BindFlags : 0,
			MiscFlags : 0,
			CPUAccessFlags : 0,
		};

        let texture_load_result = parse_dds_header( paintnet::BLACK_4X4_BC1 );

		assert_eq!( texture_load_result.is_ok(), true );

		let texture_header = texture_load_result.unwrap();

		assert_eq!( texture_header_ref.Width, texture_header.Width );
		assert_eq!( texture_header_ref.Height, texture_header.Height );
		assert_eq!( texture_header_ref.MipLevels, texture_header.MipLevels );
		assert_eq!( texture_header_ref.ArraySize, texture_header.ArraySize );
		assert_eq!( texture_header_ref.Format, texture_header.Format );
		assert_eq!( texture_header_ref.SampleDesc.Count, texture_header.SampleDesc.Count );
		assert_eq!( texture_header_ref.SampleDesc.Quality, texture_header.SampleDesc.Quality );
		assert_eq!( texture_header_ref.Usage, texture_header.Usage );
		assert_eq!( texture_header_ref.BindFlags, texture_header.BindFlags );
		assert_eq!( texture_header_ref.MiscFlags, texture_header.MiscFlags );
		assert_eq!( texture_header_ref.CPUAccessFlags, texture_header.CPUAccessFlags );
    }
}