package message;

import java.util.Objects;

public class ActionMessage extends BaseMessage {
    String type;
    String variable;
    int value;
    int comparison;
    int timestamp;
    int source;

    public ActionMessage(String type, String variable, int value, int comparison, int timestamp, int source) {
        this.comparison = comparison;
        this.variable = variable;
        this.source = source;
        this.timestamp = timestamp;
        this.type = type;
        this.value = value;
    }

    public int getTimestamp() {
        return timestamp;
    }

    public void setTimestamp(int timestamp) {
        this.timestamp = timestamp;
    }

    public int getComparison() {
        return comparison;
    }

    public void setComparison(int comparison) {
        this.comparison = comparison;
    }

    public int getSource() {
        return source;
    }

    public void setSource(int source) {
        this.source = source;
    }

    public int getValue() {
        return value;
    }

    public void setValue(int value) {
        this.value = value;
    }

    public String getType() {
        return type;
    }

    public void setType(String type) {
        this.type = type;
    }

    public void setVariable(String variable) {
        this.variable = variable;
    }

    public String getVariable() {
        return variable;
    }

    @Override
    public String toString() {
        if (Objects.equals(type, "set")) {
            return "set " + variable + " to " + value;
        }
        else {
            return  "exchange " + variable + " from " + comparison + " to " + value;
        }
    }
}
