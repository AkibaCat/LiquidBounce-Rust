// Developer oracle: original NoPose dimension constructor and real MC 26.3 methods.
import com.google.gson.*;
import java.nio.file.*;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.entity.*;
import net.minecraft.world.entity.player.Player;

public final class NoPoseDimensionsFacts {
    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        var rows = new JsonArray();
        var choices = new float[][] {{.6f,1.8f,1.54f},{.6f,1.65f,1.54f},{.6f,1.5f,1.27f}};
        for (int choice=0;choice<choices.length;choice++) {
            var c=choices[choice];
            var original=EntityDimensions.scalable(c[0],c[1]).withEyeHeight(c[2])
                .withAttachments(EntityAttachments.builder().attach(EntityAttachment.VEHICLE,Player.DEFAULT_VEHICLE_ATTACHMENT));
            for (float scale : new float[] {0f,.5f,1f,1.234567f,2f,16f}) {
                var d=original.scale(scale);
                var vehicle=d.attachments().get(EntityAttachment.VEHICLE,0,0f);
                var passenger=d.attachments().get(EntityAttachment.PASSENGER,0,0f);
                var row=new JsonObject();row.addProperty("choice",choice+1);row.addProperty("scale",scale);
                row.addProperty("widthBits",Integer.toUnsignedLong(Float.floatToRawIntBits(d.width())));
                row.addProperty("heightBits",Integer.toUnsignedLong(Float.floatToRawIntBits(d.height())));
                row.addProperty("eyeBits",Integer.toUnsignedLong(Float.floatToRawIntBits(d.eyeHeight())));
                row.addProperty("vehicleYBits",Long.toUnsignedString(Double.doubleToRawLongBits(vehicle.y)));
                row.addProperty("passengerYBits",Long.toUnsignedString(Double.doubleToRawLongBits(passenger.y)));
                row.addProperty("fixed",d.fixed());rows.add(row);
            }
        }
        var report=new JsonObject();report.add("rows",rows);
        report.addProperty("defaultVehicleYBits",Long.toUnsignedString(Double.doubleToRawLongBits(Player.DEFAULT_VEHICLE_ATTACHMENT.y)));
        Files.writeString(Path.of(args[0]),new GsonBuilder().setPrettyPrinting().create().toJson(report));
    }
}
