display_angle=46;
$fn=100;
radius=2;
radius_holder=2.1;
radius_hole=1.6;
radius_big_hole=1.7;
radius_screw_hole=1.6;
radius_empty_hole=1.1;
size_x=89.5;
size_y=110;
border=6;
thickness=2;
pcb_offset=2;
display_offset=22;

module M2(){
    translate([42.5,-2,thickness+pcb_offset])
        cube([25,4,11]);
    }
    
    //M2();
module SD(){
    translate([47.75,size_y-5,thickness+pcb_offset-2])
        cube([15.5,4,10]);
    }
    
    //SD();

module Boot_pins(){
    translate([29.00,size_y-5,thickness+pcb_offset])
        cube([17,4,17.5]);
    }
    
    //Boot_pins();

module screen (height,angle) {
    translate([21,1,pcb_offset+display_offset])
        rotate([0,-display_angle,0])
            cube ([67.5,105.5,4.5+0.5]);   
    }
    
    //screen();
module colonne(x,y){
    translate([x,y,0]){
        difference(){
            union(){
                cylinder(h=pcb_offset+thickness,d=4.2);
                cylinder(h=pcb_offset+thickness+1.6,d=3.2);    
                }
            translate([0,0,thickness]){
                cylinder(h=pcb_offset+thickness+1,d=2.2);
                }
            }
         }
      }
      
    //colonne(0,0);
module support(x,y){
    translate([x,y,0])
        cube([1,3.2,pcb_offset+thickness]);            
    }
    //support(40,40);
    
module side() {    
    translate([0,1.25,0]){
    rotate([90,0,0])
        linear_extrude(2.5)
            polygon([[20,0], [22+67.5*cos(display_angle),0],[22+67.5*cos(display_angle),display_offset+pcb_offset+67.5*sin(display_angle)],[21-4.5*sin(display_angle)+67.5*cos(display_angle)+cos(display_angle),display_offset+pcb_offset+67.5*sin(display_angle)+4.5*cos(display_angle)+sin(display_angle)],[20-4.5*sin(display_angle),display_offset+pcb_offset+4.60977*cos(display_angle+13)],[20,display_offset+pcb_offset]]);
    }
    }
    //side();
    
module holder() {
      
        translate([21+57.519562*cos(display_angle-1.4943336),-1.25,display_offset+pcb_offset+57.519562*sin(display_angle-1.4943336)]){
            rotate([0,-display_angle,0])
            union(){
             cube([11.5,size_y,6]);
             translate([-58.6,0,1]){
                cube([2,3.5,5]);}
             translate([-58.6,size_y-3.5,1]){
                cube([2,3.5,5]);
             }
          }
        }
      }
    
    //holder();
module triangle1(x,y,z){
    translate([x,y,z]){
      rotate([0,90-display_angle,0]){
       linear_extrude(57.5){
         polygon([[0,0],[0,1.5],[2.5,0]]);        
    }}}}
    //triangle(21,1.2,display_offset+pcb_offset,0);

module triangle2(x,y,z){
    translate([x,y,z]){
      rotate([0,90-display_angle,0]){
       linear_extrude(57.5){
         polygon([[0,0],[0,-1.5],[2.5,0]]);        
    }}}}    

module eth(){    
    translate([10.25,size_y-5,thickness+pcb_offset])
        cube([17,4,16]);    
    }
    //eth();
    
module usb(){    
    translate([19.5,-2,pcb_offset+thickness-1])
        cube([11,4,8]);   
    }
    //usb();
    
//Screen holder
difference(){
    union(){
        side();
        holder();
        triangle1(21,1.2,display_offset+pcb_offset);
        triangle2(21,size_y-3.7,display_offset+pcb_offset);
        translate([0,size_y-2.5,0]){
                side();       
        }
        }
    union(){    
        screen();
        SD();
        Boot_pins();
        M2();
        eth();
        usb();
    }
}
//


// Main socle
translate([0, -1.25, 0]) {
    difference() {
        translate([radius, radius, 0]) {
            minkowski()
            {
                cube([size_x-2*radius,size_y-2*radius,thickness/2]);
                cylinder(r=radius,h=thickness/2);
            }
        }
        translate([border+radius, border+radius, -0.5]) {
            minkowski()
            {
                cube([size_x-2*radius-2*border,size_y-2*radius-2*border,thickness+0.6]);
                cylinder(r=radius,h=2);
            }
        }
    }
   
}

//colonnes
colonne(3.5, 65);
colonne(3.5, 7);
colonne(69.5, 102.5);
colonne(69.5, 6);
colonne(3.5,102.5);

//supports
support(41,0);
support(32,0);
support(69,1);
support(40,size_y-6);
support(31,size_y-6);
support(69,size_y-6);









