use anyhow::{bail, Result};
use lightningcss::properties::Property;
use lightningcss::printer::PrinterOptions;
use lightningcss::cssparser::RGBA;

#[derive(Debug, Default)]
pub struct Corners<T> where T: Default {
  nw: (T, T),
  ne: (T, T),
  sw: (T, T),
  se: (T, T),
}

#[derive(Debug, Default)]
pub struct Sides<T> where T: Default {
  top: T,
  right: T,
  bottom: T,
  left: T,
}

#[derive(Debug, Default)]
pub struct ComputedProperties {
  border_radius: Corners<f32>,
  border: Sides<(Option<RGBA>, f32)>,
  margin: Sides<f32>,
  padding: Sides<f32>,
  background_color: Option<RGBA>,
}

impl ComputedProperties {
  pub fn apply(&mut self, p: &Property) -> Result<()> {
    use Property as P;
    use lightningcss::values::length::LengthPercentageOrAuto::LengthPercentage;
    use lightningcss::values::length::LengthValue::Px;
    use lightningcss::values::length::Length;
    use lightningcss::values::size::Size2D;
    use lightningcss::values::percentage::DimensionPercentage::Dimension;
    use lightningcss::properties::margin_padding::{Padding, Margin};
    use lightningcss::properties::border::BorderColor;
    use lightningcss::properties::border::BorderSideWidth;
    use lightningcss::properties::border::BorderWidth;
    use lightningcss::properties::border::GenericBorder;
    use lightningcss::properties::border_radius::BorderRadius;

    match p {
      P::PaddingTop(LengthPercentage(Dimension(Px(v)))) => self.padding.top = *v,
      P::PaddingBottom(LengthPercentage(Dimension(Px(v)))) => self.padding.bottom = *v,
      P::PaddingRight(LengthPercentage(Dimension(Px(v)))) => self.padding.right = *v,
      P::PaddingLeft(LengthPercentage(Dimension(Px(v)))) => self.padding.left = *v,
      P::Padding(Padding {
        top: LengthPercentage(Dimension(Px(t))),
        bottom: LengthPercentage(Dimension(Px(b))),
        right: LengthPercentage(Dimension(Px(r))),
        left: LengthPercentage(Dimension(Px(l))),
      }) => {
        self.padding.top = *t;
        self.padding.bottom = *b;
        self.padding.left = *l;
        self.padding.right = *r;
      },
      P::MarginTop(LengthPercentage(Dimension(Px(v)))) => self.margin.top = *v,
      P::MarginBottom(LengthPercentage(Dimension(Px(v)))) => self.margin.bottom = *v,
      P::MarginRight(LengthPercentage(Dimension(Px(v)))) => self.margin.right = *v,
      P::MarginLeft(LengthPercentage(Dimension(Px(v)))) => self.margin.left = *v,
      P::Margin(Margin {
        top: LengthPercentage(Dimension(Px(t))),
        bottom: LengthPercentage(Dimension(Px(b))),
        right: LengthPercentage(Dimension(Px(r))),
        left: LengthPercentage(Dimension(Px(l))),
      }) => {
        self.margin.top = *t;
        self.margin.bottom = *b;
        self.margin.left = *l;
        self.margin.right = *r;
      },
      P::BorderTopColor(c) => self.border.top.0 = Some(c.into()),
      P::BorderBottomColor(c) => self.border.bottom.0 = Some(c.into()),
      P::BorderLeftColor(c) => self.border.left.0 = Some(c.into()),
      P::BorderRightColor(c) => self.border.right.0 = Some(c.into()),
      P::BorderTopWidth(BorderSideWidth::Length(Length::Value(Px(v)))) => self.border.top.1 = *v,
      P::BorderBottomWidth(BorderSideWidth::Length(Length::Value(Px(v)))) => self.border.bottom.1 = *v,
      P::BorderLeftWidth(BorderSideWidth::Length(Length::Value(Px(v)))) => self.border.left.1 = *v,
      P::BorderRightWidth(BorderSideWidth::Length(Length::Value(Px(v)))) => self.border.right.1 = *v,
      P::BorderWidth(BorderWidth {
        top: BorderSideWidth::Length(Length::Value(Px(t))),
        bottom: BorderSideWidth::Length(Length::Value(Px(b))),
        right: BorderSideWidth::Length(Length::Value(Px(r))),
        left: BorderSideWidth::Length(Length::Value(Px(l))),
      }) => {
        self.border.top.1 = *t;
        self.border.bottom.1 = *b;
        self.border.left.1 = *l;
        self.border.right.1 = *r;
      },
      P::BorderColor(BorderColor {
        top: t,
        bottom: b,
        right: r,
        left: l,
      }) => {
        self.border.top.0 = Some(t.into());
        self.border.bottom.0 = Some(b.into());
        self.border.left.0 = Some(l.into());
        self.border.right.0 = Some(r.into());
      },
      P::Border(GenericBorder {
        style: _,
        width: BorderSideWidth::Length(Length::Value(Px(w))),
        color: c,
      }) => {
        let v = (Some(c.into()), *w);
        self.border.top = v.clone();
        self.border.bottom = v.clone();
        self.border.left = v.clone();
        self.border.right = v.clone();
      }
      P::BackgroundColor(c) => {
        self.background_color = Some(c.into());
      },
      P::BorderTopLeftRadius(Size2D(Dimension(Px(a)), Dimension(Px(b))), _) => self.border_radius.nw = (*a, *b),
      P::BorderTopRightRadius(Size2D(Dimension(Px(a)), Dimension(Px(b))), _) => self.border_radius.ne = (*a, *b),
      P::BorderBottomLeftRadius(Size2D(Dimension(Px(a)), Dimension(Px(b))), _) => self.border_radius.sw = (*a, *b),
      P::BorderBottomRightRadius(Size2D(Dimension(Px(a)), Dimension(Px(b))), _) => self.border_radius.se = (*a, *b),
      P::BorderRadius(BorderRadius {
        top_left: Size2D(Dimension(Px(nwa)), Dimension(Px(nwb))),
        top_right: Size2D(Dimension(Px(nea)), Dimension(Px(neb))),
        bottom_left: Size2D(Dimension(Px(swa)), Dimension(Px(swb))),
        bottom_right: Size2D(Dimension(Px(sea)), Dimension(Px(seb))),
      }, _) => {
        self.border_radius.nw = (*nwa, *nwb);
        self.border_radius.ne = (*nea, *neb);
        self.border_radius.sw = (*swa, *swb);
        self.border_radius.se = (*sea, *seb);
      },
      _ => {
        let o = PrinterOptions::default();
        match p.to_css_string(false, o) {
          Ok(css) => bail!("Unsupported property: {}", css),
          Err(e) => bail!("Unexpected error: {}", e),
        }
      }
    }
    Ok(())
  }
}
