                    Primitive::Item{stack,origin,scale}=>{
                        let owner=self.player_owner(Some(&stack));
                        let state=self.resolve_item(&stack,horizon_assets::item_model::DisplayContext::Gui,Some(&owner),0);
                        let first=batch.vertices.len();
                        horizon_client::item_render::gui::draw(&state,&self.pack.item_renderer,&self.gui.atlas.sprites,&self.item_host(),batch,origin[0],origin[1],self.glint_frame.millis/1000.,self.glint_frame.speed,self.glint_frame.alpha);
                        for v in &mut batch.vertices[first..]{v.pos=[at[0]+v.pos[0]*scale,at[1]+v.pos[1]*scale];}
                    }
                    Primitive::LiquidBounceText{text,origin,scale,color,shadow}=>{
                        use horizon_client::hud::Rect;
                        let(ascent,_)=self.gui.atlas.font.liquidbounce_target_metrics();
                        for is_shadow in[true,false]{if is_shadow&&!shadow{continue;}let offset=if is_shadow{2.*scale}else{0.};let mut x=at[0]+origin[0]+offset;let y=at[1]+origin[1]+ascent*scale+offset;
                            for span in &text.spans{for c in span.text.chars(){let g=self.gui.atlas.font.liquidbounce_target_glyph(c,span.style.bold==Some(true));
                                let fill=native_color(if is_shadow{let a=(color>>24).min(150);(a<<24)}else{color});
                                if let Some(Rect{u0,v0,u1,v1,..})=g.rect{let x0=x+g.left*scale;let x1=x+g.right*scale;let y0=y+g.top*scale;let y1=y+g.bottom*scale;batch.quad([[x0,y0],[x1,y0],[x1,y1],[x0,y1]],[[u0,v0],[u1,v0],[u1,v1],[u0,v1]],fill);}x+=g.advance*scale;
                            }}
                        }
                    }
